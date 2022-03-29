use core::ops::{Index, IndexMut};

use x86_64::PhysAddr;

use crate::{
    allocator::FrameAllocator,
    memory::get_offset,
    paging::{Page, PageTableEntry, PageTableEntryFlags, PageTableIndex, Phys},
    virt_addr::VirtAddr,
};

const PAGE_TABLE_SIZE: usize = 512;

#[repr(align(4096))]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PageTable {
    entries: [PageTableEntry; PAGE_TABLE_SIZE],
}

impl PageTable {
    pub fn new() -> Self {
        PageTable {
            entries: [PageTableEntry::new_zero(); PAGE_TABLE_SIZE],
        }
    }

    /// Load a page table from a physical frame address
    ///
    /// This is unsafe because it transmutes the start address of the frame into a page table
    /// If it doesn't actually point to a page table memory corruption could occur
    #[inline]
    pub unsafe fn load_table<'a>(frame: Phys) -> &'a PageTable {
        let virt = get_offset() + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();

        &*table_ptr
    }

    /// Load a mutable page table from a physical frame address
    ///
    /// This is unsafe because it transmutes the start address of the frame into a page table
    /// If it doesn't actually point to a page table memory corruption could occur
    /// Calling it twice with the same frame will create aliased references
    #[inline]
    pub unsafe fn load_mut_table<'a>(frame: Phys) -> &'a mut PageTable {
        let virt = get_offset() + frame.start_address().as_u64();
        let table_ptr: *mut PageTable = virt.as_mut_ptr();

        &mut *table_ptr
    }

    /// Translate a virtual address into a physical one
    pub fn translate_addr(&self, addr: VirtAddr) -> Option<PhysAddr> {
        let mut table = self;
        let mut phys_addr = PhysAddr::new(0);

        for i in 0..4 {
            let level = 3 - i;
            let index = addr.page_table_index(level);

            match table[index].frame(level) {
                Some(f) => match f {
                    Phys::Size2Mb(_) | Phys::Size1Gb(_) => {
                        phys_addr = f.start_address();
                        break;
                    }
                    _ => {
                        table = unsafe { PageTable::load_table(f) };
                        phys_addr = f.start_address();
                    }
                },
                None => return None,
            }
        }

        Some(phys_addr + u64::from(addr.page_offset()))
    }

    /// Create a new page table mapping using allocator to allocate new page table frames
    /// as required
    ///
    /// This is unsafe because if we map to an existing frame
    /// we can create aliased mutable references
    pub unsafe fn map_page<T: FrameAllocator>(
        &mut self,
        page: Page,
        entry: PageTableEntry,
        allocator: &mut T,
    ) -> Result<(), PageMapError> {
        self.map_page_inner(page, entry, allocator)
    }

    // TODO: Allow huge page mapping
    // TODO: Handle huge pages properly
    // TODO: Handle page table entry flushing correctly
    #[inline]
    fn map_page_inner<T: FrameAllocator>(
        &mut self,
        page: Page,
        new_entry: PageTableEntry,
        allocator: &mut T,
    ) -> Result<(), PageMapError> {
        let addr = page.as_virt_addr();

        let mut table = self;

        for i in 0..3 {
            let level = 3 - i;
            let index = addr.page_table_index(level);

            match table[index].frame(level) {
                Some(f) => match f {
                    Phys::Size2Mb(_) | Phys::Size1Gb(_) => {
                        // Set the table entry here so we can index the correct virtual address PTE level
                        if table[addr.page_table_index(level)]
                            .flags()
                            .contains(PageTableEntryFlags::PRESENT)
                        {
                            return Err(PageMapError::PageAlreadyMapped);
                        }
                        table[addr.page_table_index(level)] = new_entry;
                        return Ok(());
                    }
                    Phys::Size4Kb(_) => {
                        table = unsafe { PageTable::load_mut_table(f) };
                    }
                },
                None => {
                    let new_frame = allocator.allocate();
                    match new_frame {
                        Some(f) => {
                            // TODO: Ensure memory is cleared
                            let entry = PageTableEntry::new(
                                f,
                                PageTableEntryFlags::PRESENT | PageTableEntryFlags::WRITABLE,
                            );
                            table[index] = entry;
                            table = unsafe { PageTable::load_mut_table(Phys::Size4Kb(f)) };
                        }
                        None => return Err(PageMapError::FrameAllocation),
                    }
                }
            }
        }

        if table[addr.page_table_index(0)]
            .flags()
            .contains(PageTableEntryFlags::PRESENT)
        {
            return Err(PageMapError::PageAlreadyMapped);
        }

        table[addr.page_table_index(0)] = new_entry;
        Ok(())
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl Index<PageTableIndex> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: PageTableIndex) -> &Self::Output {
        &self.entries[usize::from(index)]
    }
}

impl IndexMut<usize> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl IndexMut<PageTableIndex> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: PageTableIndex) -> &mut Self::Output {
        &mut self.entries[usize::from(index)]
    }
}

// TODO: Parameterize with page size
#[derive(Debug)]
pub enum PageMapError {
    FrameAllocation,
    PageAlreadyMapped,
}

// TODO: Add huge page tests
#[cfg(test)]
mod tests {
    use x86_64::{
        structures::paging::{PhysFrame, Size4KiB},
        PhysAddr,
    };

    use crate::{
        allocator::{ZeroAllocator, FRAME_ALLOCATOR},
        paging::{Page, PageTableEntry, PageTableEntryFlags},
        vga_buffer::VGA_BUFFER_ADDRESS,
        virt_addr::VirtAddr,
    };

    use super::{PageMapError, PageTable};

    #[test_case]
    fn get_unmapped_address() {
        let table = PageTable::new();
        let addr = VirtAddr::new(1234);
        let phys_addr = table.translate_addr(addr);

        match phys_addr {
            Some(pa) => panic!("{:?} was mapped to {} unexpectedly", addr, pa.as_u64()),
            None => (),
        };
    }

    #[test_case]
    fn add_valid_entry() {
        let mut table = PageTable::new();
        let addr = VirtAddr::new(0xDEADBEEF);
        let page = Page::containing_address(addr);
        let frame = PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(4096)).unwrap();
        let entry = PageTableEntry::new(frame, PageTableEntryFlags::PRESENT);

        let alloc = match FRAME_ALLOCATOR.wait() {
            Some(a) => a,
            None => panic!("boot info allocator not initialized"),
        };

        let result = unsafe { table.map_page(page, entry, &mut *alloc.lock()) };
        match result {
            Ok(_) => {}
            Err(err) => panic!("error mapping page: {:?}", err),
        }

        let phys_addr = table.translate_addr(addr);

        match phys_addr {
            Some(pa) => assert_eq!(pa.as_u64(), addr.page_offset().as_u64() + 4096),
            None => panic!("new page was not mapped to correct physical frame"),
        }
    }

    #[test_case]
    fn try_to_remap() {
        let mut table = PageTable::new();
        let addrs = [VirtAddr::new(VGA_BUFFER_ADDRESS)];
        for addr in addrs {
            let page = Page::containing_address(addr);
            let frame = PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(0)).unwrap();
            let entry = PageTableEntry::new(frame, PageTableEntryFlags::PRESENT);

            let result = unsafe { table.map_page(page, entry, &mut ZeroAllocator {}) };
            match result {
                Ok(_) => panic!("page should not be remapped"),
                Err(PageMapError::PageAlreadyMapped) => {}
                Err(err) => panic!("error mapping page: {:?}", err),
            }
        }
    }
}
