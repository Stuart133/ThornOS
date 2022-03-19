use core::ops::Index;

use bitflags::bitflags;
use x86_64::{
    structures::paging::{PhysFrame, Size1GiB, Size2MiB, Size4KiB},
    PhysAddr,
};

const PAGE_TABLE_SIZE: usize = 512;

#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    entries: [PageTableEntry; PAGE_TABLE_SIZE],
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

/// Guaranteed to hold only values from 0..4096
#[derive(Debug)]
#[repr(transparent)]
pub struct PageOffset(u16);

impl PageOffset {
    #[inline]
    pub fn new_truncate(offset: u16) -> PageOffset {
        PageOffset(offset % 4096)
    }
}

impl From<PageOffset> for u16 {
    #[inline]
    fn from(index: PageOffset) -> Self {
        index.0
    }
}

impl From<PageOffset> for u64 {
    #[inline]
    fn from(index: PageOffset) -> Self {
        index.0 as u64
    }
}

/// Guaranteed to hold only values from 0..512
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PageTableIndex(u16);

impl PageTableIndex {
    /// Create a new index, truncating all but the lower 9 bits
    #[inline]
    pub fn new_truncate(index: u16) -> PageTableIndex {
        PageTableIndex(index % 512)
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

#[derive(Debug)]
pub enum Phys {
    Size4Kb(PhysFrame<Size4KiB>),
    Size2Mb(PhysFrame<Size2MiB>),
    Size1Gb(PhysFrame<Size1GiB>),
}

bitflags! {
    pub struct PageTableEntryFlags: u64 {
        const PRESENT = 1;
        const WRITABLE = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const DISABLE_CACHE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
        const NO_EXECUTE = 1 << 63;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn frame(self, level: usize) -> Option<Phys> {
        if !self.flags().contains(PageTableEntryFlags::PRESENT) {
            return None;
        }

        if self.flags().contains(PageTableEntryFlags::HUGE_PAGE) {
            match level {
                1 => Some(Phys::Size1Gb(PhysFrame::<Size1GiB>::containing_address(
                    self.addr(),
                ))),
                2 => Some(Phys::Size2Mb(PhysFrame::<Size2MiB>::containing_address(
                    self.addr(),
                ))),
                _ => panic!("huge page mapped at level {}", level + 1),
            }
        } else {
            Some(Phys::Size4Kb(PhysFrame::containing_address(self.addr())))
        }
    }

    #[inline]
    fn addr(self) -> PhysAddr {
        PhysAddr::new(self.0 & 0x000F_FFFF_FFFF_F000)
    }

    #[inline]
    pub fn flags(self) -> PageTableEntryFlags {
        PageTableEntryFlags::from_bits_truncate(self.0)
    }
}

impl Phys {
    pub fn start_address(self) -> PhysAddr {
        match self {
            Phys::Size4Kb(f) => f.start_address(),
            Phys::Size2Mb(f) => f.start_address(),
            Phys::Size1Gb(f) => f.start_address(),
        }
    }
}

impl From<PhysFrame> for Phys {
    fn from(p: PhysFrame) -> Self {
        Phys::Size4Kb(p)
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

#[cfg(test)]
mod tests {
    use super::{PageOffset, PageTableIndex};

    #[test_case]
    fn page_table_index_truncate() {
        let index = 1234;
        let pti = PageTableIndex::new_truncate(index);

        assert_eq!(u16::from(pti), 210);
    }

    #[test_case]
    fn page_offset_truncate() {
        let offset = 12345;
        let offset = PageOffset::new_truncate(offset);

        assert_eq!(u16::from(offset), 57);
    }
}
