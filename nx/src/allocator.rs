use core::{alloc::GlobalAlloc, ffi::c_void};

// Link malloc and free from libnx
extern "C" {
    pub fn malloc(size: usize) -> *mut c_void;
    pub fn free(ptr: *mut c_void);
}

pub struct NxAllocator;

// Currently ignoring the layout, hope this will be fine lol
unsafe impl GlobalAlloc for NxAllocator{
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let ptr = malloc(layout.size());
        return ptr as *mut u8;
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        free(ptr as *mut c_void);
    }
}