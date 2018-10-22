#![feature(ptr_internals)]
#![feature(panic_implementation)]
#![feature(lang_items)]
#![feature(const_fn)]
#![feature(alloc)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(alloc_error_handler)]
#![no_std]

extern crate hole_list_allocator as allocator;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate alloc;
#[macro_use]
extern crate lazy_static;
extern crate multiboot2;
#[macro_use]
extern crate once;
extern crate rlibc;
extern crate spin;
extern crate volatile;
#[macro_use]
extern crate x86_64;
extern crate bit_field;
extern crate cpuio;

#[macro_use]
mod vga_buffer;
mod drivers;
mod interrupts;
mod memory;
mod pic;

pub const HEAP_START: usize = 0o_000_001_000_000_0000; // heap starts at the second P3 entry
pub const HEAP_SIZE: usize = 100 * 1024; // 100KiB

#[global_allocator]
static GLOBAL_ALLOC: allocator::Allocator = allocator::Allocator;

#[no_mangle]
pub extern "C" fn rust_main(multiboot_information_address: usize) {
    use memory::FrameAllocator;

    vga_buffer::clear_screen();

    let boot_info = unsafe { multiboot2::load(multiboot_information_address) };
    enable_nxe_bit();
    enable_write_protect_bit();

    // remap the kernel, set up the guard page and map the heap pages
    let mut memory_controller = memory::init(&boot_info);

    unsafe {
        interrupts::init(&mut memory_controller);
    }

    unsafe {
        asm!("sti");
    }

    println!("It did not crash!");
    loop {}
}

fn enable_nxe_bit() {
    use x86_64::registers::msr::{rdmsr, wrmsr, IA32_EFER};

    let nxe_bit = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe_bit);
    }
}

fn enable_write_protect_bit() {
    use x86_64::registers::control_regs::{cr0, cr0_write, Cr0};

    unsafe { cr0_write(cr0() | Cr0::WRITE_PROTECT) };
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

use core::panic::PanicInfo;

#[panic_implementation]
#[no_mangle]
pub fn panic(panic_info: &PanicInfo) -> ! {
    unsafe {
        if let Some(location) = panic_info.location() {
            println!(
                "\n\nPANIC in {} at line {}:",
                location.file(),
                location.line()
            );
            loop {}
        } else {
            println!("\n\nPANIC but can't get location information.");
            loop {}
        }
    }
}

#[alloc_error_handler]
fn alloc_error(_: core::alloc::Layout) -> ! {
    panic!("Alloc error!");
}
