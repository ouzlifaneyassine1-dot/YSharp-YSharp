// ---------------------------------------------------------------------------
// OY# High-Performance ECS — Archetype-based, SoA layout, cache-line aware
// Designed for ultra-realistic simulations with millions of entities.
// ---------------------------------------------------------------------------

use core::alloc::Layout;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU32, Ordering};

// ---------------------------------------------------------------------------
// Type-erased component storage (SoA)
// ---------------------------------------------------------------------------

pub type ComponentId = u32;
pub type ArchetypeId = u16;
pub type EntityId = u32;

static NEXT_ENTITY: AtomicU32 = AtomicU32::new(1);

/// A type-erased sparse array of component data in SoA layout.
/// Each chunk holds `CHUNK_CAP` entities in a contiguous array.
pub struct ComponentChunk {
    data: NonNull<u8>,
    layout: Layout,
    count: u32,
    capacity: u32,
}

const CHUNK_CAP: u32 = 64; // entities per chunk (cache-line friendly)

impl ComponentChunk {
    pub fn new(element_size: usize, align: usize) -> Self {
        let cap = CHUNK_CAP as usize;
        let layout = Layout::from_size_align(cap * element_size, align).unwrap();
        let data = unsafe { std::alloc::alloc(layout) };
        ComponentChunk {
            data: NonNull::new(data).expect("OOM: component chunk"),
            layout,
            count: 0,
            capacity: CHUNK_CAP,
        }
    }

    #[inline(always)]
    pub unsafe fn read<T: Copy>(&self, index: u32) -> T { unsafe {
        let ptr = self.data.as_ptr().add(index as usize * core::mem::size_of::<T>());
        core::ptr::read(ptr as *const T)
    }}

    #[inline(always)]
    pub unsafe fn write<T>(&self, index: u32, value: T) { unsafe {
        let ptr = self.data.as_ptr().add(index as usize * core::mem::size_of::<T>());
        core::ptr::write(ptr as *mut T, value);
    }}

    #[inline(always)]
    pub fn has_space(&self) -> bool { self.count < self.capacity }

    pub fn push(&mut self) -> u32 {
        let idx = self.count;
        self.count += 1;
        idx
    }
}

impl Drop for ComponentChunk {
    fn drop(&mut self) {
        unsafe { std::alloc::dealloc(self.data.as_ptr(), self.layout); }
    }
}

// ---------------------------------------------------------------------------
// Archetype: a unique combination of component types
// ---------------------------------------------------------------------------

pub struct Archetype {
    pub id: ArchetypeId,
    pub components: Vec<ComponentId>,
    /// SoA storage: chunks[component_index] -> Vec<chunks>
    chunks: Vec<Vec<ComponentChunk>>,
    /// Entity IDs per slot
    entities: Vec<EntityId>,
    count: u32,
}

impl Archetype {
    pub fn new(id: ArchetypeId, components: Vec<ComponentId>, sizes: &[usize], aligns: &[usize]) -> Self {
        let num_components = components.len();
        let mut chunks = Vec::with_capacity(num_components);
        for i in 0..num_components {
            let mut comp_chunks = Vec::new();
            comp_chunks.push(ComponentChunk::new(sizes[i], aligns[i]));
            chunks.push(comp_chunks);
        }
        Archetype { id, components, chunks, entities: Vec::new(), count: 0 }
    }

    #[inline(always)]
    pub fn entity_count(&self) -> u32 { self.count }

    /// Add an entity to this archetype. Returns the slot index.
    pub fn add_entity(&mut self, entity: EntityId) -> u32 {
        let slot = self.count;
        if slot as usize >= self.entities.len() {
            self.entities.push(entity);
        } else {
            self.entities[slot as usize] = entity;
        }
        // Ensure all component chunks have capacity
        for comp_chunks in self.chunks.iter_mut() {
            let has_space = comp_chunks.last().map_or(false, |c| c.has_space());
            if !has_space {
                let size = core::mem::size_of::<f32>(); // placeholder; should be stored per component
                comp_chunks.push(ComponentChunk::new(size, 16));
            }
            comp_chunks.last_mut().unwrap().push();
        }
        self.count += 1;
        slot
    }

    /// Get a raw pointer to component data for a given component type at slot.
    #[inline(always)]
    pub unsafe fn component_data<T: Copy>(&self, component_idx: usize, slot: u32) -> T { unsafe {
        let chunk_idx = (slot / CHUNK_CAP) as usize;
        let in_chunk = slot % CHUNK_CAP;
        self.chunks[component_idx][chunk_idx].read::<T>(in_chunk)
    }}

    #[inline(always)]
    pub unsafe fn set_component<T>(&self, component_idx: usize, slot: u32, value: T) { unsafe {
        let chunk_idx = (slot / CHUNK_CAP) as usize;
        let in_chunk = slot % CHUNK_CAP;
        self.chunks[component_idx][chunk_idx].write(in_chunk, value);
    }}

    /// Iterate over all entities in this archetype.
    /// The callback receives (entity_id, slot_index) for each live entity.
    pub fn for_each<F: FnMut(EntityId, u32)>(&self, mut f: F) {
        for slot in 0..self.count {
            f(self.entities[slot as usize], slot);
        }
    }

    /// Parallel-friendly: returns slices for chunked iteration.
    pub fn chunk_count(&self) -> usize {
        ((self.count + CHUNK_CAP - 1) / CHUNK_CAP) as usize
    }
}

// ---------------------------------------------------------------------------
// World — top-level ECS container
// ---------------------------------------------------------------------------

pub struct World {
    archetypes: Vec<Archetype>,
    entity_to_slot: Vec<(ArchetypeId, u32)>, // entity -> (archetype, slot)
    component_sizes: Vec<usize>,
    component_aligns: Vec<usize>,
}

impl World {
    pub fn new() -> Self {
        World {
            archetypes: Vec::new(),
            entity_to_slot: Vec::new(),
            component_sizes: Vec::new(),
            component_aligns: Vec::new(),
        }
    }

    pub fn register_component<T: 'static>(&mut self) -> ComponentId {
        let id = self.component_sizes.len() as ComponentId;
        self.component_sizes.push(core::mem::size_of::<T>());
        self.component_aligns.push(core::mem::align_of::<T>());
        id
    }

    pub fn find_or_create_archetype(&mut self, components: &[ComponentId]) -> ArchetypeId {
        // Check existing
        for arch in &self.archetypes {
            if arch.components.as_slice() == components {
                return arch.id;
            }
        }
        let id = self.archetypes.len() as ArchetypeId;
        let sizes: Vec<usize> = components.iter().map(|&c| self.component_sizes[c as usize]).collect();
        let aligns: Vec<usize> = components.iter().map(|&c| self.component_aligns[c as usize]).collect();
        self.archetypes.push(Archetype::new(id, components.to_vec(), &sizes, &aligns));
        id
    }

    pub fn spawn(&mut self, components: &[ComponentId]) -> EntityId {
        let entity = NEXT_ENTITY.fetch_add(1, Ordering::Relaxed);
        let arch_id = self.find_or_create_archetype(components);
        let slot = self.archetypes[arch_id as usize].add_entity(entity);
        if (entity as usize) >= self.entity_to_slot.len() {
            self.entity_to_slot.resize(entity as usize + 256, (0, 0));
        }
        self.entity_to_slot[entity as usize] = (arch_id, slot);
        entity
    }

    /// Query all entities with a given set of components.
    /// This is the main iteration primitive — cache-friendly chunk traversal.
    pub fn query<F: FnMut(EntityId)>(&self, components: &[ComponentId], mut f: F) {
        for arch in &self.archetypes {
            if components.iter().all(|c| arch.components.contains(c)) {
                arch.for_each(|entity, _slot| f(entity));
            }
        }
    }

    /// Write a component value for an entity.
    pub fn set<T: Copy>(&self, entity: EntityId, component_id: ComponentId, value: T) {
        if let Some(&(arch_id, slot)) = self.entity_to_slot.get(entity as usize) {
            let arch = &self.archetypes[arch_id as usize];
            if let Some(comp_idx) = arch.components.iter().position(|&c| c == component_id) {
                unsafe { arch.set_component(comp_idx, slot, value); }
            }
        }
    }

    /// Read a component value for an entity.
    pub fn get<T: Copy>(&self, entity: EntityId, component_id: ComponentId) -> Option<T> {
        self.entity_to_slot.get(entity as usize).and_then(|&(arch_id, slot)| {
            let arch = &self.archetypes[arch_id as usize];
            arch.components.iter().position(|&c| c == component_id).map(|comp_idx| {
                unsafe { arch.component_data::<T>(comp_idx, slot) }
            })
        })
    }

    pub fn archetype_count(&self) -> usize { self.archetypes.len() }
    pub fn total_entities(&self) -> u32 {
        self.archetypes.iter().map(|a| a.entity_count()).sum()
    }
}

// ---------------------------------------------------------------------------
// System — operates on a query and runs per-frame
// ---------------------------------------------------------------------------

pub trait System {
    fn run(&mut self, world: &World, dt: f32);
}

pub struct SystemManager {
    systems: Vec<Box<dyn System>>,
}

impl SystemManager {
    pub fn new() -> Self { SystemManager { systems: Vec::new() } }
    pub fn add<S: System + 'static>(&mut self, system: S) { self.systems.push(Box::new(system)); }
    pub fn run_all(&mut self, world: &World, dt: f32) {
        for sys in &mut self.systems {
            sys.run(world, dt);
        }
    }
}
