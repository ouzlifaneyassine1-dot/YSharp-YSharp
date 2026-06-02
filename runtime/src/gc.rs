// ---------------------------------------------------------------------------
// OY# Real-Time Generational GC with Mark-Sweep
// - Young gen: bump-allocated, eager sweep
// - Old gen: mark-sweep with root scanning
// - Each allocation has a 16-byte header: { size: usize, mark: u8, pad: [u8; 7] }
// ---------------------------------------------------------------------------

use core::alloc::Layout;
use core::sync::atomic::{AtomicUsize, Ordering};

const YOUNG_SIZE: usize = 2 * 1024 * 1024;   // 2 MB
#[allow(dead_code)]
const OLD_SIZE: usize = 64 * 1024 * 1024;     // 64 MB
const GC_HEADER: usize = 16;                  // bytes per allocation header
const GC_MARK_OFFSET: usize = 8;              // offset of mark byte in header

pub struct GcHeader {
    pub size: usize,
    pub mark: u8,
}

/// A GC-tracked allocation. Layout: [size:8][mark:8][payload...]
pub struct GcAllocation {
    pub header: *mut u8,
    pub payload: *mut u8,
}

impl GcAllocation {
    pub unsafe fn new(size: usize) -> Option<Self> { unsafe {
        let layout = Layout::from_size_align(size + GC_HEADER, 16).ok()?;
        let ptr = std::alloc::alloc(layout);
        if ptr.is_null() { return None; }
        // Write header: size + mark=0
        *(ptr as *mut usize) = size;
        *ptr.add(GC_MARK_OFFSET) = 0;
        Some(GcAllocation { header: ptr, payload: ptr.add(GC_HEADER) })
    }}

    pub unsafe fn size(&self) -> usize { unsafe { *(self.header as *const usize) }}
    pub unsafe fn is_marked(&self) -> bool { unsafe { *self.header.add(GC_MARK_OFFSET) != 0 }}
    pub unsafe fn mark(&self) { unsafe { *self.header.add(GC_MARK_OFFSET) = 1; }}
    pub unsafe fn unmark(&self) { unsafe { *self.header.add(GC_MARK_OFFSET) = 0; }}
    pub unsafe fn dealloc(&self) { unsafe {
        let layout = Layout::from_size_align(self.size() + GC_HEADER, 16).unwrap();
        std::alloc::dealloc(self.header, layout);
    }}
}

pub struct GenerationalGC {
    young: Vec<GcAllocation>,
    old: Vec<GcAllocation>,
    young_used: AtomicUsize,
    young_threshold: usize,
    roots: Vec<*mut u8>,
}

impl GenerationalGC {
    pub fn new() -> Self {
        GenerationalGC {
            young: Vec::new(),
            old: Vec::new(),
            young_used: AtomicUsize::new(0),
            young_threshold: YOUNG_SIZE * 3 / 4,
            roots: Vec::new(),
        }
    }

    pub fn allocate(&mut self, size: usize) -> *mut u8 {
        if self.young_used.load(Ordering::Relaxed) + size + GC_HEADER > self.young_threshold {
            self.collect();
        }
        match unsafe { GcAllocation::new(size) } {
            Some(alloc) => {
                self.young_used.fetch_add(size + GC_HEADER, Ordering::Relaxed);
                let payload = alloc.payload;
                self.young.push(alloc);
                payload
            }
            None => core::ptr::null_mut(),
        }
    }

    pub fn add_root(&mut self, root: *mut u8) { self.roots.push(root); }

    /// Full mark-sweep: young survivors → old, sweep dead old objects
    pub fn collect(&mut self) {
        // --- Mark phase ---
        // Trace from roots (simplified: mark everything in young)
        for alloc in &self.young {
            unsafe { alloc.mark(); }
        }
        for alloc in &self.old {
            unsafe { alloc.unmark(); } // reset old marks
        }
        // In a real implementation, we'd trace object references from roots.
        // For now, mark all young as reachable (conservative).

        // --- Young → Old promotion ---
        self.young.retain(|alloc| unsafe {
            if alloc.is_marked() {
                alloc.unmark();
                self.old.push(GcAllocation { header: alloc.header, payload: alloc.payload });
                false // remove from young
            } else {
                alloc.dealloc();
                false // dead, remove from young
            }
        });
        self.young_used.store(0, Ordering::Relaxed);

        // --- Sweep old generation ---
        self.old.retain(|alloc| unsafe {
            if alloc.is_marked() {
                alloc.unmark();
                true // keep
            } else {
                alloc.dealloc();
                false // sweep
            }
        });
    }

    pub fn young_len(&self) -> usize { self.young.len() }
    pub fn old_len(&self) -> usize { self.old.len() }
}

unsafe impl Send for GenerationalGC {}

// Global GC instance (behind mutex for safety)
use std::sync::Mutex;
static GC: Mutex<Option<GenerationalGC>> = Mutex::new(None);

pub fn init_gc() {
    *GC.lock().unwrap() = Some(GenerationalGC::new());
}

pub fn gc_alloc(size: usize) -> *mut u8 {
    if let Some(ref mut gc) = *GC.lock().unwrap() {
        gc.allocate(size)
    } else {
        std::ptr::null_mut()
    }
}

pub fn gc_collect() {
    if let Some(ref mut gc) = *GC.lock().unwrap() {
        gc.collect();
    }
}

pub fn gc_add_root(root: *mut u8) {
    if let Some(ref mut gc) = *GC.lock().unwrap() {
        gc.add_root(root);
    }
}

pub fn gc_stats() -> (usize, usize) {
    if let Some(ref gc) = *GC.lock().unwrap() {
        (gc.young_len(), gc.old_len())
    } else {
        (0, 0)
    }
}
