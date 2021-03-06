use memory::MemoryController;
use pic::ChainedPics;
use spin::{Mutex, Once};

use x86_64::instructions::port::inb;
use x86_64::structures::idt::{ExceptionStackFrame, Idt};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtualAddress;

mod gdt;

static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(0x20, 0x28) });
const DOUBLE_FAULT_IST_INDEX: usize = 0;
static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<gdt::Gdt> = Once::new();

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();

        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);
        }

        idt.breakpoint.set_handler_fn(breakpoint_handler);

        for i in 0..224 {
            if i == 1 {
                idt.interrupts[i].set_handler_fn(keyboard_handler);
            } else {
                idt.interrupts[i].set_handler_fn(dummy_handler);
            }
        }

        idt
    };
}

pub unsafe fn init(memory_controller: &mut MemoryController) {
    PICS.lock().init();
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;
    use x86_64::structures::gdt::SegmentSelector;
    let double_fault_stack = memory_controller
        .alloc_stack(1)
        .expect("cold not allocate double fault stack");

    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] =
            VirtualAddress(double_fault_stack.top());
        tss
    });

    let mut code_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);

    let gdt = GDT.call_once(|| {
        let mut gdt = gdt::Gdt::new();
        code_selector = gdt.add_entry(gdt::Descriptor::kernel_code_segment());
        tss_selector = gdt.add_entry(gdt::Descriptor::tss_segment(&tss));
        gdt
    });

    gdt.load();

    set_cs(code_selector);
    load_tss(tss_selector);

    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
) {
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn keyboard_handler(stack_frame: &mut ExceptionStackFrame) {
    use drivers::keyboard::read_scancode_from_keyboard;

    if let Some(input) = read_scancode_from_keyboard() {
        use vga_buffer::backspace;
        if input == '\x7F' {
            backspace();
        } else {
            print!("{}", input);
        }
    }
    unsafe {
        PICS.lock().notify_end_of_interrupt(0x21 as u8);
    }
}

extern "x86-interrupt" fn dummy_handler(stack_frame: &mut ExceptionStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(0x20 as u8);
    }
}
