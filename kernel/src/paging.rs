use core::ops::Add;

use bitflags::bitflags;
use x86_64::{
    structures::paging::{PageSize, PhysFrame, Size1GiB, Size2MiB, Size4KiB},
    PhysAddr,
};

use crate::virt_addr::VirtAddr;

/// Guaranteed to hold only values from 0..4096
#[derive(Debug)]
#[repr(transparent)]
pub struct PageOffset(u16);

impl PageOffset {
    #[inline]
    pub fn new_truncate(offset: u16) -> PageOffset {
        PageOffset(offset % 4096)
    }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0 as u64
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

#[derive(Debug, Clone, Copy)]
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
    // TODO - Validate huge page flag with passed page size
    pub fn new<S: PageSize>(frame: PhysFrame<S>, flags: PageTableEntryFlags) -> Self {
        PageTableEntry(frame.start_address().as_u64() | flags.bits)
    }

    // TODO: Better name for this
    pub fn new_zero() -> Self {
        PageTableEntry(0)
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Page(VirtAddr);

// TODO: Parameterize pages with size
impl Page {
    #[inline]
    pub fn containing_address(addr: VirtAddr) -> Self {
        Page(addr.align_down())
    }

    #[inline]
    pub fn as_virt_addr(self) -> VirtAddr {
        self.0
    }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0.as_u64()
    }
}

impl Add<u64> for Page {
    type Output = Page;

    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        Page::containing_address(self.0 + 4096 * rhs)
    }
}

pub struct PageRangeInclusive {
    start: Page,
    end: Page,
}

impl PageRangeInclusive {
    pub fn new(start: Page, end: Page) -> Self {
        PageRangeInclusive {
            start: start,
            end: end,
        }
    }
}

impl Iterator for PageRangeInclusive {
    type Item = Page;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let p = Some(self.start);
            self.start = self.start + 1;
            p
        } else {
            None
        }
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
    use x86_64::{
        structures::paging::{PhysFrame, Size2MiB, Size4KiB},
        PhysAddr,
    };

    use crate::virt_addr::VirtAddr;

    use super::{
        Page, PageOffset, PageRangeInclusive, PageTableEntry, PageTableEntryFlags, PageTableIndex,
    };

    #[test_case]
    fn add_4kb_page() {
        let addr = VirtAddr::new(4096);
        let page = Page::containing_address(addr);

        assert_eq!((page + 5).as_u64(), 24_576);
    }

    #[test_case]
    fn iterate_inclusive_page_range() {
        let start_page = Page::containing_address(VirtAddr::new(0));
        let end_page = Page::containing_address(VirtAddr::new(20_000));
        let page_range = PageRangeInclusive::new(start_page, end_page);

        let mut c = 0;
        for page in page_range {
            assert_eq!(page.as_u64(), start_page.as_u64() + c * 4096);
            c += 1;
        }

        assert_eq!(c, 4);
    }

    #[test_case]
    fn iterate_reverse_inclusive_page_range() {
        let start_page = Page::containing_address(VirtAddr::new(20_000));
        let end_page = Page::containing_address(VirtAddr::new(0));
        let page_range = PageRangeInclusive::new(start_page, end_page);

        for _ in page_range {
            panic!("empty range shouldn't produce any iteration");
        }
    }

    #[test_case]
    fn iterate_empty_inclusive_page_range() {
        let page = Page::containing_address(VirtAddr::new(0));
        let page_range = PageRangeInclusive::new(page, page);

        for _ in page_range {
            panic!("empty range shouldn't produce any iteration");
        }
    }

    #[test_case]
    fn unmapped_page_returns_none() {
        let pte = PageTableEntry::new(
            PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(0)),
            PageTableEntryFlags { bits: 0 },
        );

        let frame = pte.frame(0);
        match frame {
            Some(_) => panic!("unmapped frame was returned"),
            None => {}
        };
    }

    #[test_case]
    fn mapped_page_returns_frame() {
        let pte = PageTableEntry::new(
            PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(4096)),
            PageTableEntryFlags::PRESENT,
        );

        let frame = pte.frame(0);
        match frame {
            Some(f) => {
                assert_eq!(f.start_address().as_u64(), 4096)
            }
            None => panic!("frame was not returned"),
        };
    }

    // TODO: Test huge page panics when panicking tests are supported
    #[test_case]
    fn mapped_hugepage_returns_frame() {
        let pte = PageTableEntry::new(
            PhysFrame::<Size2MiB>::containing_address(PhysAddr::new(8000)),
            PageTableEntryFlags::PRESENT | PageTableEntryFlags::HUGE_PAGE,
        );

        let frame = pte.frame(1);
        match frame {
            Some(f) => {
                assert_eq!(f.start_address().as_u64(), 0)
            }
            None => panic!("frame was not returned"),
        };
    }

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
