// ---------------------------------------------------------------------------
// OY# Physics Engine — rigid body dynamics, broadphase (SAP), narrowphase
// Designed for cache-friendly simulation of 10k+ bodies.
// ---------------------------------------------------------------------------

use crate::simd_math::*;

// ---------------------------------------------------------------------------
// RigidBody
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RigidBody {
    pub position: float4,
    pub rotation: Quat,
    pub velocity: float4,
    pub angular_vel: float4,
    pub mass: f32,
    pub inv_mass: f32,
    pub inertia: float4x4,
    pub inv_inertia: float4x4,
    pub restitution: f32,
    pub friction: f32,
    pub is_static: bool,
    pub shape: ShapeType,
    pub aabb: AABB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ShapeType { Sphere, Box, Capsule, Plane, Mesh }

impl RigidBody {
    pub fn new_sphere(pos: float4, radius: f32, mass: f32) -> Self {
        let inv = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        let inertia = float4x4::identity() * (0.4 * mass * radius * radius);
        let half = float4(radius, radius, radius, 0.0);
        RigidBody {
            position: pos, rotation: Quat::identity(),
            velocity: float4::zero(), angular_vel: float4::zero(),
            mass, inv_mass: inv, inertia,
            inv_inertia: if mass > 0.0 { float4x4::identity() * (1.0 / (0.4 * mass * radius * radius)) } else { float4x4::zero() },
            restitution: 0.5, friction: 0.3, is_static: mass <= 0.0,
            shape: ShapeType::Sphere,
            aabb: AABB::from_center(pos, half),
        }
    }

    pub fn new_box(pos: float4, half_ext: float4, mass: f32) -> Self {
        let inv = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        let (x2, y2, z2) = (half_ext.0 * half_ext.0, half_ext.1 * half_ext.1, half_ext.2 * half_ext.2);
        let i = 0.3333 * mass;
        let inertia = float4x4([
            float4(i * (y2 + z2), 0.0, 0.0, 0.0),
            float4(0.0, i * (x2 + z2), 0.0, 0.0),
            float4(0.0, 0.0, i * (x2 + y2), 0.0),
            float4(0.0, 0.0, 0.0, 1.0),
        ]);
        RigidBody {
            position: pos, rotation: Quat::identity(),
            velocity: float4::zero(), angular_vel: float4::zero(),
            mass, inv_mass: inv, inertia: inertia.clone(),
            inv_inertia: if mass > 0.0 { float4x4::identity() * (1.0 / (i * (y2 + z2).max(1e-10))) } else { float4x4::zero() },
            restitution: 0.3, friction: 0.5, is_static: mass <= 0.0,
            shape: ShapeType::Box,
            aabb: AABB::from_center(pos, half_ext),
        }
    }

    /// Integrate forces (gravity) and update velocity
    pub fn apply_gravity(&mut self, gravity: float4, dt: f32) {
        if self.is_static { return; }
        self.velocity = self.velocity + gravity * dt;
    }

    /// Integrate position and rotation
    pub fn integrate(&mut self, dt: f32) {
        if self.is_static { return; }
        self.position = self.position + self.velocity * dt;
        // Update AABB
        let half = (self.aabb.max - self.aabb.min) * 0.5;
        self.aabb = AABB::from_center(self.position, half);
    }
}

// ---------------------------------------------------------------------------
// Broadphase — Sweep and Prune (SAP) along X axis
// O(n log n) sort + O(n) sweep. Cache-friendly array layout.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SweepEntry {
    pub id: u32,
    pub min: f32,
    pub max: f32,
    pub is_start: bool,
}

pub struct Broadphase {
    pub entries: Vec<SweepEntry>,
    pub pairs: Vec<(u32, u32)>,
}

impl Broadphase {
    pub fn new(capacity: usize) -> Self {
        Broadphase { entries: Vec::with_capacity(capacity * 2), pairs: Vec::with_capacity(capacity * 4) }
    }

    /// Build sweep entries from rigid bodies, detect overlapping pairs.
    pub fn sweep(&mut self, bodies: &[RigidBody]) {
        self.entries.clear();
        self.pairs.clear();

        for (i, body) in bodies.iter().enumerate() {
            let id = i as u32;
            self.entries.push(SweepEntry { id, min: body.aabb.min.0, max: body.aabb.max.0, is_start: true });
            self.entries.push(SweepEntry { id, min: body.aabb.min.0, max: body.aabb.max.0, is_start: false });
        }

        // Sort by min value (stable)
        self.entries.sort_by(|a, b| {
            a.min.partial_cmp(&b.min).unwrap_or(core::cmp::Ordering::Equal)
                .then_with(|| a.is_start.cmp(&b.is_start).reverse())
        });

        // Sweep
        let mut active: Vec<u32> = Vec::with_capacity(64);
        for entry in &self.entries {
            if entry.is_start {
                for &active_id in &active {
                    if active_id != entry.id {
                        self.pairs.push((active_id, entry.id));
                    }
                }
                active.push(entry.id);
            } else {
                active.retain(|&id| id != entry.id);
            }
        }
    }

    pub fn pair_count(&self) -> usize { self.pairs.len() }
}

// ---------------------------------------------------------------------------
// Narrowphase — sphere-sphere and box-box collision
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Contact {
    pub entity_a: u32,
    pub entity_b: u32,
    pub point: float4,
    pub normal: float4,
    pub penetration: f32,
}

impl Contact {
    pub fn new(a: u32, b: u32, point: float4, normal: float4, penetration: f32) -> Self {
        Contact { entity_a: a, entity_b: b, point, normal, penetration }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CollisionResult {
    pub hit: bool,
    pub point: float4,
    pub normal: float4,
    pub penetration: f32,
}

impl CollisionResult {
    pub const fn miss() -> Self { CollisionResult { hit: false, point: float4::zero(), normal: float4::zero(), penetration: 0.0 } }
}

/// Sphere vs sphere test
pub fn sphere_sphere(a: float4, ra: f32, b: float4, rb: f32) -> CollisionResult {
    let delta = b - a;
    let dist_sq = delta.length_sq();
    let r_sum = ra + rb;
    if dist_sq >= r_sum * r_sum || dist_sq < 1e-10 {
        return CollisionResult::miss();
    }
    let dist = dist_sq.sqrt();
    CollisionResult {
        hit: true,
        point: a + delta * (ra / dist),
        normal: delta * (1.0 / dist),
        penetration: r_sum - dist,
    }
}

/// AABB vs AABB test
pub fn aabb_vs_aabb(a: &AABB, b: &AABB) -> bool { a.intersects(b) }

/// Box vs box (SAT-based, simplified)
pub fn box_box(a: &RigidBody, b: &RigidBody) -> CollisionResult {
    if !a.aabb.intersects(&b.aabb) {
        return CollisionResult::miss();
    }
    // Simple penetration along minimum separating axis
    let centers = b.position - a.position;
    let half_a = a.aabb.half_extents();
    let half_b = b.aabb.half_extents();
    let overlap = half_a + half_b - centers.abs();
    let min_overlap = overlap.0.min(overlap.1).min(overlap.2);
    if min_overlap <= 0.0 { return CollisionResult::miss(); }

    let normal = if overlap.0 <= overlap.1 && overlap.0 <= overlap.2 {
        float4(centers.0.signum(), 0.0, 0.0, 0.0)
    } else if overlap.1 <= overlap.2 {
        float4(0.0, centers.1.signum(), 0.0, 0.0)
    } else {
        float4(0.0, 0.0, centers.2.signum(), 0.0)
    };
    CollisionResult { hit: true, point: a.position + normal * half_a.dot(normal.abs()), normal, penetration: min_overlap }
}

// ---------------------------------------------------------------------------
// Constraint solver — sequential impulse
// ---------------------------------------------------------------------------

pub fn resolve_contacts(bodies: &mut [RigidBody], contacts: &[Contact], _dt: f32) {
    for contact in contacts {
        let a_idx = contact.entity_a as usize;
        let b_idx = contact.entity_b as usize;
        if a_idx >= bodies.len() || b_idx >= bodies.len() { continue; }
        let (left, right) = bodies.split_at_mut(b_idx);
        let body_a = &mut left[a_idx];
        let body_b = &mut right[0];

        let rel_vel = body_b.velocity - body_a.velocity;
        let rel_normal = rel_vel.dot(contact.normal);
        if rel_normal > 0.0 { continue; }

        let e = body_a.restitution.min(body_b.restitution);
        let j = -(1.0 + e) * rel_normal / (body_a.inv_mass + body_b.inv_mass);
        let impulse = contact.normal * j;

        body_a.velocity = body_a.velocity - impulse * body_a.inv_mass;
        body_b.velocity = body_b.velocity + impulse * body_b.inv_mass;

        // Friction
        let tangent = (rel_vel - contact.normal * rel_normal).normalize();
        let friction_force = tangent * j.abs() * body_a.friction.min(body_b.friction);
        body_a.velocity = body_a.velocity + friction_force * body_a.inv_mass;
        body_b.velocity = body_b.velocity - friction_force * body_b.inv_mass;
    }
}

// ---------------------------------------------------------------------------
// PhysicsWorld — ties it all together
// ---------------------------------------------------------------------------

pub struct PhysicsWorld {
    pub bodies: Vec<RigidBody>,
    pub gravity: float4,
    broadphase: Broadphase,
    pub contacts: Vec<Contact>,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        PhysicsWorld {
            bodies: Vec::new(),
            gravity: float4(0.0, -9.81, 0.0, 0.0),
            broadphase: Broadphase::new(1024),
            contacts: Vec::new(),
        }
    }

    pub fn add_body(&mut self, body: RigidBody) -> u32 {
        let id = self.bodies.len() as u32;
        self.bodies.push(body);
        id
    }

    /// Main simulation step
    pub fn step(&mut self, dt: f32) {
        // 1. Apply forces
        for body in &mut self.bodies {
            body.apply_gravity(self.gravity, dt);
        }

        // 2. Integrate
        for body in &mut self.bodies {
            body.integrate(dt);
        }

        // 3. Broadphase
        self.broadphase.sweep(&self.bodies);
        self.contacts.clear();

        // 4. Narrowphase
        for &(a, b) in &self.broadphase.pairs {
            let body_a = &self.bodies[a as usize];
            let body_b = &self.bodies[b as usize];
            let result = match (body_a.shape, body_b.shape) {
                (ShapeType::Sphere, ShapeType::Sphere) => {
                    sphere_sphere(body_a.position, 1.0, body_b.position, 1.0)
                }
                (ShapeType::Box, ShapeType::Box) => box_box(body_a, body_b),
                _ => CollisionResult::miss(),
            };
            if result.hit {
                self.contacts.push(Contact::new(a, b, result.point, result.normal, result.penetration));
            }
        }

        // 5. Solve constraints
        resolve_contacts(&mut self.bodies, &self.contacts, dt);
    }

    pub fn body_count(&self) -> usize { self.bodies.len() }
    pub fn contact_count(&self) -> usize { self.contacts.len() }
}
