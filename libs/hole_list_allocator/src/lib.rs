#![feature(const_fn)]
#![feature(allocator_api)]
#![feature(alloc)]
#![feature(global_allocator)]
#![no_std]
#![deny(warnings)]

extern crate alloc;
extern crate spin;
extern crate linked_list_allocator;

use alloc::heap::{Alloc, AllocErr, Layout};
use spin::Mutex;
use linked_list_allocator::Heap;


static HEAP: Mutex<Option<Heap>> = Mutex::new(None);

pub unsafe fn init(offset: usize, size: usize) {
    *HEAP.lock() = Some(Heap::new(offset, size));
}

pub struct Allocator;

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.allocate_first_fit(layout)
        } else {
            panic!("Heap not initialized!");
        }
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.deallocate(ptr, layout)
        } else {
            panic!("heap not initalized");
        }
    }
}

#[global_allocator]
static GLOBAL_ALLOC: Allocator = Allocator;
