pub use self::entry::*;
pub use self::mapper::Mapper;

use memory::{PAGE_SIZE, Frame, FrameAllocator};
use self::table::{Table, Level4};
use self::temporary_page::TemporaryPage;

use core::ptr::Unique;
use core::ops::{Deref, DerefMut};
use x86_64::instructions::tlb;
use x86_64::registers::control_regs;

mod entry;
mod table;
mod temporary_page;
mod mapper;

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress  = usize;

#[derive(Debug, Clone, Copy)]
pub struct Page {
    number: usize,
}

impl Page {
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(address < 0x000_8000_0000_000 || address >= 0xffff_8000_000_000,
                "invalid address: 0x{:x}", address);
        Page { number: address / PAGE_SIZE }
    }

    fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }
}

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn with<F>(&mut self, table: &mut InactivePageTable, f: F) where F: FnOnce(&mut Mapper)
    {
        {
        let backup = Frame::containing_address(unsafe { control_regs::cr3() } as usize);
        let p4_table = temporary_page.map_table_frame(backup.clone(), self);
        self.p4_mut()[511].set(table.p4_frame.clone(), PRESENT | WRITABLE);
        tlb::flush_all();
        f(self);
        p4_table[511
        } // inner scope ensures the table variable is dropped before unmapping the temporary page.
        temporary_page.unmap(self);
    }
}

pub struct InactivePageTable {
    p4_frame: Frame
}

impl InactivePageTable {
    pub fn new(frame: Frame, active_table: &mut ActivePageTable, temporary_page: &mut TemporaryPage) -> InactivePageTable
    {
        {
            let table = temporary_page.map_table_frame(frame.clone(), active_table);
            table.zero();
            table[511].set(frame.clone(), PRESENT | WRITABLE);
        } // inner scope ensures the table variable is dropped before unmapping the temporary page.
        temporary_page.unmap(active_table);
        InactivePageTable { p4_frame: frame }
    }
}
