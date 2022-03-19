use core::ops::Add;

use crate::paging::{PageOffset, PageTableIndex};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl VirtAddr {
    // TODO: Make this canonical
    pub fn new(addr: u64) -> VirtAddr {
        VirtAddr(addr)
    }

    #[inline]
    pub fn page_offset(&self) -> PageOffset {
        PageOffset::new_truncate(self.0 as u16)
    }

    #[inline]
    pub fn page_table_index(&self, level: usize) -> PageTableIndex {
        let index = (self.0 >> (12 + level * 9)) as u16;
        PageTableIndex::new_truncate(index)
    }

    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    #[inline]
    pub fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    #[inline]
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }
}

impl Add<u64> for VirtAddr {
    type Output = VirtAddr;

    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        VirtAddr::new(self.0 + rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::VirtAddr;

    #[test_case]
    fn get_page_offset() {
        let addr = VirtAddr::new(0xE677_BF54_D244);
        let level1: u16 = addr.page_offset().into();
        assert_eq!(level1, 580);
    }

    #[test_case]
    fn get_level1_index() {
        let addr = VirtAddr::new(0xE677_BF54_D244);
        let level1: u16 = addr.page_table_index(0).into();
        assert_eq!(level1, 333);
    }

    #[test_case]
    fn get_level2_index() {
        let addr = VirtAddr::new(0xE677_BF54_D244);
        let level1: u16 = addr.page_table_index(1).into();
        assert_eq!(level1, 506);
    }

    #[test_case]
    fn get_level3_index() {
        let addr = VirtAddr::new(0xE677_BF54_D244);
        let level1: u16 = addr.page_table_index(2).into();
        assert_eq!(level1, 478);
    }

    #[test_case]
    fn get_level4_index() {
        let addr = VirtAddr::new(0xE677_BF54_D244);
        let level1: u16 = addr.page_table_index(3).into();
        assert_eq!(level1, 460);
    }
}
