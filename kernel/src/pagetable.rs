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
#[derive(Debug)]
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
    #[inline]
    unsafe fn load_mut_table<'a>(frame: Phys) -> &'a mut PageTable {
        let virt = get_offset() + frame.start_address().as_u64();
        let table_ptr: *mut PageTable = virt.as_mut_ptr();

        &mut *table_ptr
    }

    pub fn translate_addr(&self, addr: VirtAddr) -> Option<PhysAddr> {
        let mut table = self;
        let mut frame: Phys;

        for i in 0..4 {
            let level = 3 - i;
            let index = addr.page_table_index(level);

            // Convert the frame into a page table reference
            // let table = unsafe { load_table(frame) };
            let entry = table[index];
            match entry.frame(level) {
                Some(f) => match f {
                    Phys::Size2Mb(_) | Phys::Size1Gb(_) => {
                        frame = f;
                        break;
                    }
                    _ => {
                        table = unsafe { PageTable::load_table(f) };
                        frame = f;
                    }
                },
                None => return None,
            }
        }

        Some(frame.start_address() + u64::from(addr.page_offset()))
    }

    // TODO: Move these to a page table impl
    /// Translate a virtual address into a physical one

    /// Create a new page table mapping using allocator to allocate new page table frames
    /// as required
    ///
    /// This is unsafe because if we map to an existing frame
    /// we can create aliased mutable references
    pub unsafe fn map_page<T: FrameAllocator>(
        &self,
        page: Page,
        entry: PageTableEntry,
        allocator: &mut T,
    ) -> Result<(), PageMapError> {
        self.map_page_inner(page, entry, allocator)
    }

    // TODO: Move these to a page table impl
    // TODO: Allow huge page mapping
    // TODO: Handle huge pages properly
    #[inline]
    fn map_page_inner<T: FrameAllocator>(
        &self,
        page: Page,
        new_entry: PageTableEntry,
        allocator: &mut T,
    ) -> Result<(), PageMapError> {
        let addr = page.as_virt_addr();

        let mut frame: Phys;
        let mut table = self;

        for i in 0..4 {
            let level = 3 - i;
            let index = addr.page_table_index(level);

            let entry = table[index];
            match entry.frame(level) {
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
                    _ => {
                        table = unsafe { PageTable::load_mut_table(frame) };
                        frame = f
                    }
                },
                None => {
                    if level != 0 {
                        let new_frame = allocator.allocate();
                        match new_frame {
                            Some(f) => {
                                // TODO: Ensure memory is cleared
                                let entry = PageTableEntry::new(
                                    f,
                                    PageTableEntryFlags::PRESENT | PageTableEntryFlags::WRITABLE,
                                );
                                table[index] = entry;
                                frame = Phys::Size4Kb(f);
                                table = unsafe { PageTable::load_mut_table(frame) };
                            }
                            None => return Err(PageMapError::FrameAllocation),
                        }
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

// TODO: Use a fresh pagetable in these tests
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
        virt_addr::VirtAddr, memory::load_active_pagetable,
    };

    use super::{get_offset, PageMapError};

    // We know the VGA buffer is identity mapped by the bootloader
    #[test_case]
    fn translate_vga_address() {
        let table = load_active_pagetable();
        let addr = VirtAddr::new(VGA_BUFFER_ADDRESS);
        let phys_addr = table.translate_addr(addr);

        match phys_addr {
            Some(pa) => assert_eq!(addr.as_u64(), pa.as_u64()),
            None => panic!("vga virtual address was not mapped"),
        };
    }

    // We know that physical address 0 is mapped & uses huge pages (This could be flaky down the line)
    #[test_case]
    fn translate_address_0() {
        // Physical Address 0 is at the map offset + 0
        let table = load_active_pagetable();
        let addr = get_offset();
        let phys_addr = table.translate_addr(addr);

        match phys_addr {
            Some(pa) => assert_eq!(pa.as_u64(), 0),
            None => panic!("physical memory was not mapped"),
        };
    }

    #[test_case]
    fn translate_missing_address() {
        let table = load_active_pagetable();
        let addr = VirtAddr::new(0xDEADBEEF);
        let phys_addr = table.translate_addr(addr);

        match phys_addr {
            Some(pa) => panic!("0xDEADBEEF was mapped to {} unexpectedly", pa.as_u64()),
            None => (),
        };
    }

    #[test_case]
    fn add_valid_entry() {
      let table = load_active_pagetable();
        let addr = VirtAddr::new(5);
        let page = Page::containing_address(addr);
        let frame = PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(4096)).unwrap();
        let entry = PageTableEntry::new(frame, PageTableEntryFlags::PRESENT);

        let result = unsafe { table.map_page(page, entry, &mut ZeroAllocator {}) };
        match result {
            Ok(_) => {}
            Err(err) => panic!("error mapping page: {:?}", err),
        }

        let phys_addr = table.translate_addr(addr);

        match phys_addr {
            Some(pa) => assert_eq!(pa.as_u64(), 4101),
            None => panic!("new page was not mapped to correct physical frame"),
        }
    }

    #[test_case]
    fn try_to_remap() {
      let table = load_active_pagetable();
        let addrs = [get_offset(), VirtAddr::new(VGA_BUFFER_ADDRESS)];
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

    #[test_case]
    fn add_allocation_entry() {
      let table = load_active_pagetable();
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
}
