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

/// Get the currently active pagetable from the cr3 register
///
/// This is unsafe as it can create aliased references if the active
/// pagetable is referenced anywhere else
pub unsafe fn load_active_pagetable<'a>() -> &'a mut PageTable {
    let (page_table, _) = Cr3::read();
    let frame = page_table.into();

    PageTable::load_mut_table(frame) // This is safe as the physical address has been loaded directly from cr3
}

/// Get a copy of the currently active pagetable from the cr3 register
pub fn copy_active_pagetable() -> PageTable {
    let (page_table, _) = Cr3::read();
    let frame = page_table.into();

    unsafe { PageTable::load_table(frame).clone() } // This is safe as the physical address has been loaded directly from cr3
}
