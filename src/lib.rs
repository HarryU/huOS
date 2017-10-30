#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![no_std]

extern crate rlibc;

mod vga_buffer;

#[no_mangle]
pub extern fn rust_main() {
    let hello = b"Hello World!";
    let colour_byte = 0x1f;

    let mut hello_coloured = [colour_byte; 24];
    for (i, char_byte) in hello.into_iter().enumerate() {
        hello_coloured[i*2] = *char_byte;
    }

    let buffer_ptr = (0xb8000 + 1988) as *mut _;
    unsafe { *buffer_ptr = hello_coloured };

    loop{}
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] #[no_mangle] pub extern fn panic_fmt() -> ! {loop{}}
