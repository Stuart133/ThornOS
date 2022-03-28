use spin::Once;
use x86_64::registers::control::Cr3;

use crate::pagetable::PageTable;
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

#[inline]
pub fn get_offset() -> VirtAddr {
    match PHYSICAL_OFFSET.wait() {
        Some(offset) => VirtAddr::new(*offset),
        None => panic!("virtual memory system is not initialized"),
    }
}

// TODO - Mark this unsafe and write description
pub fn load_active_pagetable<'a>() -> &'a PageTable {
    let (page_table, _) = Cr3::read();
    let frame = page_table.into();

    unsafe { PageTable::load_table(frame) }
}
