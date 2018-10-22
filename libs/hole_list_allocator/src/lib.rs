#![feature(const_fn)]
#![feature(allocator_api)]
#![feature(alloc)]
#![no_std]
#![deny(warnings)]

extern crate alloc;
extern crate linked_list_allocator;
extern crate spin;

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;
use linked_list_allocator::Heap;
use spin::Mutex;

static HEAP: Mutex<Option<Heap>> = Mutex::new(None);

pub unsafe fn init(offset: usize, size: usize) {
    *HEAP.lock() = Some(Heap::new(offset, size));
}

pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.allocate_first_fit(layout).unwrap().as_ptr()
        } else {
            panic!("Heap not initialized!");
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.deallocate(NonNull::new(ptr).unwrap(), layout)
        } else {
            panic!("heap not initalized");
        }
    }
}
