// ---------------------------------------------------------------------------
// OY# Frame Allocator — zero-GC, O(1) per-frame allocation
// Uses double-buffered bump allocators that reset every frame.
// Ideal for per-frame transient data in games and simulations.
// ---------------------------------------------------------------------------

use core::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;

const FRAME_POOL_SIZE: usize = 64 * 1024 * 1024; // 64 MB per frame pool
const ALIGN: usize = 16;

struct FramePool {
    base: usize,
    current: AtomicUsize,
    capacity: usize,
}

impl FramePool {
    const fn new() -> Self {
        FramePool { base: 0, current: AtomicUsize::new(0), capacity: 0 }
    }

    unsafe fn init(&mut self, base: usize, capacity: usize) {
        self.base = base;
        self.current.store(base, Ordering::SeqCst);
        self.capacity = capacity;
    }

    fn alloc(&self, size: usize) -> *mut u8 {
        let aligned = (size + ALIGN - 1) & !(ALIGN - 1);
        let prev = self.current.fetch_add(aligned, Ordering::Relaxed);
        if prev + aligned > self.base + self.capacity {
            return core::ptr::null_mut();
        }
        prev as *mut u8
    }

    fn reset(&self) {
        self.current.store(self.base, Ordering::Relaxed);
    }

    fn used(&self) -> usize {
        self.current.load(Ordering::Relaxed) - self.base
    }
}

pub struct FrameAllocator {
    current: UnsafeCell<usize>, // 0 or 1 — which pool is active
    pools: [FramePool; 2],
    committed: bool,
}

impl FrameAllocator {
    pub const fn new() -> Self {
        FrameAllocator {
            current: UnsafeCell::new(0),
            pools: [FramePool::new(), FramePool::new()],
            committed: false,
        }
    }

    pub unsafe fn init(&mut self) { unsafe {
        for (_i, pool) in self.pools.iter_mut().enumerate() {
            let base = alloc_os_pages(FRAME_POOL_SIZE).expect("frame alloc: OS allocation failed");
            pool.init(base as usize, FRAME_POOL_SIZE);
        }
        self.committed = true;
    }}

    #[inline(always)]
    pub fn alloc(&self, size: usize) -> *mut u8 {
        if !self.committed { return core::ptr::null_mut(); }
        let idx = unsafe { *self.current.get() };
        self.pools[idx].alloc(size)
    }

    #[inline(always)]
    pub fn reset(&self) {
        if !self.committed { return; }
        let idx = unsafe { *self.current.get() };
        self.pools[idx].reset();
    }

    /// Swap to the other pool and reset the new active one.
    /// Call once per frame.
    pub fn end_frame(&self) {
        if !self.committed { return; }
        let idx = unsafe { *self.current.get() };
        let next = idx ^ 1;
        self.pools[next].reset();
        unsafe { *self.current.get() = next; }
    }

    pub fn stats(&self) -> (usize, usize) {
        let idx = unsafe { *self.current.get() };
        (self.pools[idx].used(), self.pools[idx ^ 1].used())
    }
}

#[cfg(target_os = "windows")]
unsafe fn alloc_os_pages(size: usize) -> Option<*mut u8> { unsafe {
    unsafe extern "system" {
        fn VirtualAlloc(lpAddress: *mut core::ffi::c_void, dwSize: usize,
                        flAllocationType: u32, flProtect: u32) -> *mut core::ffi::c_void;
    }
    const MEM_RESERVE_COMMIT: u32 = 0x2000 | 0x1000;
    const PAGE_READWRITE: u32 = 0x04;
    let ptr = VirtualAlloc(core::ptr::null_mut(), size, MEM_RESERVE_COMMIT, PAGE_READWRITE);
    if ptr.is_null() { None } else { Some(ptr as *mut u8) }
}}

#[cfg(not(target_os = "windows"))]
unsafe fn alloc_os_pages(size: usize) -> Option<*mut u8> {
    extern "C" {
        fn mmap(addr: *mut core::ffi::c_void, length: usize, prot: i32,
                flags: i32, fd: i32, offset: isize) -> *mut core::ffi::c_void;
    }
    const PROT_READ_WRITE: i32 = 0x01 | 0x02;
    const MAP_PRIVATE_ANON: i32 = 0x02 | 0x20;
    let ptr = mmap(core::ptr::null_mut(), size, PROT_READ_WRITE, MAP_PRIVATE_ANON, -1, 0);
    if ptr == libc::MAP_FAILED { None } else { Some(ptr as *mut u8) }
}

unsafe impl Sync for FrameAllocator {}

// Global frame allocator instance
pub static FRAME_ALLOC: FrameAllocator = FrameAllocator::new();
