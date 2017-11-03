#![feature(const_fn)]
#![feature(allocator_api)]
#![feature(alloc)]
#![feature(global_allocator)]

#![no_std]

use alloc::heap::{Alloc, AllocErr, Layout};
use spin::Mutex;

extern crate alloc;
extern crate spin;

struct LockedHeap {
    heap: Mutex<Heap>,
}

#[global_allocator]
static GLOBAL_ALLOC: LockedHeap = LockedHeap::empty();

pub unsafe fn init(start: usize, size: usize) {
    GLOBAL_ALLOC.init(start, size);
}

/// The heap is protected by the LockedHeap struct.
impl LockedHeap {
    /// Creates a protected empty heap. All allocate calls will
    /// return `AllocErr`.
    pub const fn empty() -> LockedHeap {
        LockedHeap {
            heap: Mutex::new(Heap::empty())
        }
    }

    /// Initializes the heap.
    unsafe fn init(&self, start: usize, size: usize) {
        self.heap.lock().init(start, size);
    }
}

unsafe impl<'a> Alloc for &'a LockedHeap {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        self.heap.lock().alloc(layout)
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        self.heap.lock().dealloc(ptr, layout)
    }
}

pub struct Heap {
    start: usize,
    end: usize,
    next: usize,
}

impl Heap {
    /// Creates an empty heap. `allocate` calls before an `init`
    /// will return `AllocErr`.
    pub const fn empty() -> Heap {
        Heap {
            start: 0,
            end: 0,
            next: 0,
        }
    }

    /// Initialises the heap given a start address and a size.
    /// This is unsafe. The start address must be valid, and
    /// the memory in the [start, start + size) range must be
    /// free. If either the start address, or the size are invalid,
    /// undefined behaviour can occur.
    unsafe fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.next = start;
    }

    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let alloc_start = align_up(self.next, layout.align());
        let alloc_end = alloc_start.saturating_add(layout.size());

        if alloc_end <= self.end {
            self.next = alloc_end;
            Ok(alloc_start as *mut u8)
        } else {
            Err(AllocErr::Exhausted{ request: layout })
        }
    }

    unsafe fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        // Leak all the memory ................
    }
}

pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of two");
    }
}

pub fn align_up(addr:usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}
