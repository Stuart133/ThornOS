use x86_64::{structures::paging::PhysFrame, PhysAddr};

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
#[derive(Debug)]
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

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn frame(self) -> PhysFrame {
        match PhysFrame::from_start_address(self.addr()) {
            Ok(f) => f,
            Err(_) => todo!(),
        }
    }

    fn addr(self) -> PhysAddr {
        PhysAddr::new(self.0 & 0x000F_FFFF_FFFF_F000)
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
