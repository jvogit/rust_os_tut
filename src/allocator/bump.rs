use core::{alloc::GlobalAlloc, ptr};

use super::{Locked, align_up};

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    allocated: usize,
    next: usize,
}

impl BumpAllocator {
    pub const fn empty() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            allocated: 0,
            next: 0,
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut bump = self.lock();

        // align alloc_start to layout align
        let alloc_start = align_up(bump.next, layout.size());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            return ptr::null_mut();
        }

        bump.next = alloc_end;
        bump.allocated += 1;

        alloc_start as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        let mut bump = self.lock();

        bump.allocated -= 1;

        if bump.allocated == 0 {
            bump.next = bump.heap_start;
        }
    }
}
