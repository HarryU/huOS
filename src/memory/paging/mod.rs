use memory::PAGE_SIZE;
use memory::Frame;

mod entry;

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress  = usize;

pub struct Page {
    number: usize,
}

pub fn translate(virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
    let offset = virtual_address % PAGE_SIZE;
    translate_page(Page::containing_address(virtual_address)).map(|frame| frame.number * PAGE_SIZE + offset)
}

pub fn containing_address(address: VirtualAddress) -> Page {
    assert!(address < 0x000_8000_0000_000 || address >= 0xffff_8000_000_000,
            "invalid address: 0x{:x}", address);
    Page { number: address / PAGE_SIZE }
}

fn start_address(&self) -> usize {
    self.number * PAGE_SIZE
}

fn translate_page(page: Page) -> Option<Frame> {
    use self::entry::HUGE_PAGE;

    let p3 = unsafe { &*table::P4 }.next_table(page.p4_index());

    let huge_page = || {
    }

    p3.and_then(|p3| p3.next_table(page.p3_index()))
      .and_then(|p2| p2.next_table(page.p2_index()))
      .and_then(|p1| p1.next_table(page.p1_index()))
      .or_else(huge_page)
}
