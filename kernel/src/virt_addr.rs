use crate::memory::{PageOffset, PageTableIndex};

#[derive(Debug)]
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
    pub fn level1_index(&self) -> PageTableIndex {
        let index = (self.0 >> 12) as u16;
        PageTableIndex::new_truncate(index)
    }

    #[inline]
    pub fn level2_index(&self) -> PageTableIndex {
        let index = (self.0 >> 21) as u16;
        PageTableIndex::new_truncate(index)
    }

    #[inline]
    pub fn level3_index(&self) -> PageTableIndex {
        let index = (self.0 >> 30) as u16;
        PageTableIndex::new_truncate(index)
    }

    #[inline]
    pub fn level4_index(&self) -> PageTableIndex {
        let index = (self.0 >> 39) as u16;
        PageTableIndex::new_truncate(index)
    }

    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
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
        let level1: u16 = addr.level1_index().into();
        assert_eq!(level1, 333);
    }

    #[test_case]
    fn get_level2_index() {
        let addr = VirtAddr::new(0xE677_BF54_D244);
        let level1: u16 = addr.level2_index().into();
        assert_eq!(level1, 506);
    }

    #[test_case]
    fn get_level3_index() {
        let addr = VirtAddr::new(0xE677_BF54_D244);
        let level1: u16 = addr.level3_index().into();
        assert_eq!(level1, 478);
    }

    #[test_case]
    fn get_level4_index() {
        let addr = VirtAddr::new(0xE677_BF54_D244);
        let level1: u16 = addr.level4_index().into();
        assert_eq!(level1, 460);
    }
}
