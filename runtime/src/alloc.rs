use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

#[cfg(target_os = "windows")]
unsafe extern "system" {
    fn VirtualAlloc(
        lpAddress: *mut core::ffi::c_void,
        dwSize: usize,
        flAllocationType: u32,
        flProtect: u32,
    ) -> *mut core::ffi::c_void;
}

#[cfg(target_os = "windows")]
const MEM_RESERVE: u32 = 0x2000;
#[cfg(target_os = "windows")]
const MEM_COMMIT: u32 = 0x1000;
#[cfg(target_os = "windows")]
const PAGE_READWRITE: u32 = 0x04;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    current: AtomicUsize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            current: AtomicUsize::new(0),
        }
    }

    pub unsafe fn init(&mut self, start: usize, size: usize) {
        self.heap_start = start;
        self.heap_end = start + size;
        self.current.store(start, Ordering::SeqCst);
    }

    pub fn alloc(&self, size: usize) -> *mut u8 {
        let alignment = 8;
        let aligned_size = (size + alignment - 1) & !(alignment - 1);
        let prev = self.current.fetch_add(aligned_size, Ordering::SeqCst);
        if prev + aligned_size > self.heap_end {
            return core::ptr::null_mut();
        }
        prev as *mut u8
    }

    pub fn reset(&mut self) {
        self.current.store(self.heap_start, Ordering::SeqCst);
    }
}

static ALLOCATOR: Mutex<BumpAllocator> = Mutex::new(BumpAllocator::new());

pub fn init_heap(size: usize) {
    unsafe {
        #[cfg(target_os = "windows")]
        let ptr = VirtualAlloc(core::ptr::null_mut(), size, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE) as usize;
        #[cfg(not(target_os = "windows"))]
        let ptr = libc::mmap(
            core::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        ) as usize;
        #[cfg(not(target_os = "windows"))]
        let ok = ptr != libc::MAP_FAILED as usize;
        #[cfg(target_os = "windows")]
        let ok = ptr != 0;
        if ok {
            ALLOCATOR.lock().unwrap().init(ptr, size);
        }
    }
}

pub fn alloc(size: usize) -> *mut u8 {
    ALLOCATOR.lock().unwrap().alloc(size)
}
