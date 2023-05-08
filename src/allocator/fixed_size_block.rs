use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::{self, NonNull},
};

use super::Locked;

struct ListNode {
    next: Option<&'static mut ListNode>,
}

fn list_index(layout: &Layout) -> Option<usize> {
    let req_size = layout.size().max(layout.align());

    BLOCK_SIZES.iter().position(|&s| s >= req_size)
}

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        Self {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut fixed_size_allocator = self.lock();

        match list_index(&layout) {
            Some(index) => {
                match fixed_size_allocator.list_heads[index].take() {
                    Some(node) => {
                        // a free block is available
                        fixed_size_allocator.list_heads[index] = node.next.take();
                        node as *mut ListNode as *mut u8
                    }
                    None => {
                        // must allocate a block
                        let block_size = BLOCK_SIZES[index];
                        // we decided the alignment of box is the same as size as the size is power of 2
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();

                        fixed_size_allocator.fallback_alloc(layout)
                    }
                }
            }
            None => {
                // it's too big use the fallback allocator
                fixed_size_allocator.fallback_alloc(layout)
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut fixed_size_allocator = self.lock();
        
        match list_index(&layout) {
            Some(index) => {
                assert!(layout.size() <= BLOCK_SIZES[index]);
                assert!(layout.align() <= BLOCK_SIZES[index]);

                let new_node = ListNode {
                    next: fixed_size_allocator.list_heads[index].take(),
                };
                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                fixed_size_allocator.list_heads[index] = Some(&mut *new_node_ptr)
            },
            None => {
                let ptr = NonNull::new(ptr).unwrap();

                fixed_size_allocator.fallback_allocator.deallocate(ptr, layout)
            }
        }
    }
}
