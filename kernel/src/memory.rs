use spin::Once;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::page_table::FrameError;
use x86_64::structures::paging::PageTable;
use x86_64::{PhysAddr, VirtAddr};

static PHYSICAL_OFFSET: Once<VirtAddr> = Once::new();

/// Initialize the viritual memory system
///
/// This function is unsafe as the caller must ensure the physical memory is mapped at the offset specified
/// or terrible things will happen. The other calls in this module rely on the fact that once the memory system
/// is initialized further calls are safe as long as the init call satisfied the requirements above
pub unsafe fn init(physical_memory_offset: VirtAddr) {
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
        addr.p1_index(),
    ];
    let mut frame = level_4_table_frame;

    for &index in &table_indices {
        // Convert the frame into a page table reference
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
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

#[cfg(test)]
mod tests {}
