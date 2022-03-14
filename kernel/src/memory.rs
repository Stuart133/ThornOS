use core::ops::Index;

use spin::Once;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::page_table::FrameError;
use x86_64::structures::paging::PageTable;
use x86_64::PhysAddr;

use crate::println;
use crate::virt_addr::VirtAddr;

static PHYSICAL_OFFSET: Once<u64> = Once::new();

/// Initialize the viritual memory system
///
/// This function is unsafe as the caller must ensure the physical memory is mapped at the offset specified
/// or terrible things will happen. The other calls in this module rely on the fact that once the memory system
/// is initialized further calls are safe as long as the init call satisfied the requirements above
pub unsafe fn init(physical_memory_offset: u64) {
    PHYSICAL_OFFSET.call_once(|| physical_memory_offset);
}

/// Return a mutable reference to the active page table
///
/// This is unsafe as multiple calls will cause the mutable reference to be aliased
/// TODO - Investigate safer shared reference implementations
pub unsafe fn active_level_4_table() -> &'static mut PageTable {
    let physical_memory_offset = get_offset();

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// Translate a virtual address into a physical one
pub fn translate_addr(addr: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr)
}

fn translate_addr_inner(addr: VirtAddr) -> Option<PhysAddr> {
    let physical_memory_offset = get_offset();

    let (level_4_table_frame, _) = Cr3::read();

    let table_indices = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.level1_index(),
    ];
    let mut frame = level_4_table_frame;

    for &index in &table_indices {
        println!("{:?}", frame);
        println!("Index: {:?}", index);
        // Convert the frame into a page table reference
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable2 = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}

fn get_offset() -> VirtAddr {
    match PHYSICAL_OFFSET.wait() {
        Some(offset) => *offset,
        None => panic!("virtual memory system to initialized"),
    }
}

// trait PageSize {
//     const SIZE: u64;
// }

// #[derive(Debug)]
// enum Size4KB {}

// impl PageSize for Size4KB {
//     const SIZE: u64 = 512;
// }

const PAGE_TABLE_SIZE: usize = 512;

#[repr(align(4096))]
#[repr(C)]
struct PageTable2 {
    entries: [u64; PAGE_TABLE_SIZE],
}

impl Index<usize> for PageTable2 {
    type Output = u64;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

#[derive(Debug)]
pub struct PageTableIndex(u16);

impl PageTableIndex {
    /// Create a new index, truncating all but the lower 9 bits
    #[inline]
    pub fn new_truncate(index: u16) -> PageTableIndex {
        PageTableIndex(index)
    }
}

impl From<PageTableIndex> for usize {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        usize::from(index.0)
    }
}

impl From<PageTableIndex> for u64 {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        u64::from(index.0)
    }
}

impl From<PageTableIndex> for u16 {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        index.0
    }
}

#[cfg(test)]
mod tests {
    use crate::vga_buffer::VGA_BUFFER_ADDRESS;
    use x86_64::VirtAddr;

    use super::{get_offset, translate_addr};

    // We know the VGA buffer is identity mapped by the bootloader
    #[test_case]
    fn translate_vga_address() {
        let addr = VirtAddr::new(VGA_BUFFER_ADDRESS);
        let phys_addr = translate_addr(addr);

        match phys_addr {
            Some(pa) => assert_eq!(addr.as_u64(), pa.as_u64()),
            None => panic!("vga virtual address was not mapped"),
        };
    }

    // // We know that physical address 0 is mapped & uses huge pages (This could be flaky down the line)
    #[test_case]
    fn translate_address_0() {
        // Physical Address 0 is at the map offset + 0
        let addr = get_offset();
        let phys_addr = translate_addr(addr);

        match phys_addr {
            Some(pa) => assert_eq!(pa.as_u64(), 0),
            None => panic!("physical memory was not mapped"),
        };
    }

    #[test_case]
    fn translate_missing_address() {
        let addr = VirtAddr::new(0xDEADBEEF);
        let phys_addr = translate_addr(addr);

        match phys_addr {
            Some(pa) => panic!("0xDEADBEEF was mapped to {} unexpectedly", pa.as_u64()),
            None => (),
        };
    }

    // Add entries to page table
    // Add entry to page table (different types too) and see if we can read it back
}
