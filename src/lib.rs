#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(const_unique_new)]
#![feature(alloc)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
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
mod memory;
mod interrupts;
mod pic;
mod keyboard;

pub const HEAP_START: usize = 0o_000_001_000_000_0000; // heap starts at the second P3 entry
pub const HEAP_SIZE:  usize = 100 * 1024;              // 100KiB

#[no_mangle]
pub extern "C" fn rust_main(multiboot_information_address: usize) {
    use memory::FrameAllocator;

    vga_buffer::clear_screen();

    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
    enable_nxe_bit();
    enable_write_protect_bit();

    // remap the kernel, set up the guard page and map the heap pages
    let mut memory_controller = memory::init(boot_info);

    unsafe {
        interrupts::init(&mut memory_controller);
    }

    unsafe {
        asm!("sti");
    }

    println!("It did not crash!");
    loop{}
}

fn enable_nxe_bit() {
    use x86_64::registers::msr::{IA32_EFER, rdmsr, wrmsr};

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

#[lang = "eh_personality"] extern fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("\n\nPANIC in {} at line {}:", file, line);
    println!("      {}", fmt);
    loop{}
}
