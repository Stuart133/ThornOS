use crate::memory::PageTableIndex;

#[derive(Debug)]
pub struct VirtAddr(u64);

impl VirtAddr {
    #[inline]
    pub fn level1_index(self) -> PageTableIndex {
        let index = (self.0 >> 12) as u16;

        PageTableIndex::new_truncate(index)
    }
}
