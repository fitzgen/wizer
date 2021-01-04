use std::alloc::{Layout, System};
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) struct ZeroDuringInitAllocator(AtomicBool);

impl ZeroDuringInitAllocator {
    pub const fn new() -> Self {
        ZeroDuringInitAllocator(AtomicBool::new(true))
    }

    pub fn finish_init(&self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

unsafe impl std::alloc::GlobalAlloc for ZeroDuringInitAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if self.0.load(Ordering::SeqCst) {
            std::ptr::write_bytes(ptr, 0, layout.size());
        }
        System.dealloc(ptr, layout)
    }
}
