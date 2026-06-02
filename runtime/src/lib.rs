#![cfg_attr(feature = "no-std", no_std)]

pub mod alloc;
pub mod gc;
pub mod ffi;
pub mod simd_math;
pub mod frame_alloc;
pub mod ecs;
pub mod physics;
mod tensor_ops;


#[unsafe(no_mangle)]
pub extern "C" fn oy_init(heap_size: usize, _stack_size: usize) {
    alloc::init_heap(heap_size);
    gc::init_gc();
}

#[unsafe(no_mangle)]
pub extern "C" fn oy_alloc(size: usize) -> *mut u8 {
    alloc::alloc(size)
}

#[unsafe(no_mangle)]
pub extern "C" fn oy_print_int(val: i64) {
    print(val);
}

#[unsafe(no_mangle)]
pub extern "C" fn oy_print_float(val: f64) {
    print(val);
}

#[unsafe(no_mangle)]
pub extern "C" fn oy_print_str(ptr: *const u8, len: usize) {
    let s = unsafe { core::slice::from_raw_parts(ptr, len) };
    print(core::str::from_utf8(s).unwrap_or("<invalid utf8>"));
}

#[unsafe(no_mangle)]
pub extern "C" fn oy_print_newline() {
    print("\n");
}

fn print(msg: impl core::fmt::Display) {
    #[cfg(not(feature = "no-std"))]
    {
        println!("{}", msg);
    }
    #[cfg(feature = "no-std")]
    {
    }
}

#[cfg(feature = "no-std")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {}
}
