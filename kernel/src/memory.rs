use x86_64::registers::control::Cr3;
use x86_64::structures::paging::PageTable;
use x86_64::VirtAddr;

/// Return a mutable reference to the active page table
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr  // TODO: Investigate ways to validate the offset so we can make this function safe to call
}
