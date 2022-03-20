use spin::Once;
use x86_64::registers::control::Cr3;
use x86_64::PhysAddr;

use crate::paging::{PageTable, PageTableEntry, Phys};
use crate::println;
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

/// Translate a virtual address into a physical one
pub fn translate_addr(addr: &VirtAddr) -> Option<PhysAddr> {
    let (level_4_table_frame, _) = Cr3::read();

    let mut frame: Phys = level_4_table_frame.into();
    for i in 0..4 {
        let level = 3 - i;
        let index = addr.page_table_index(level);

        // Convert the frame into a page table reference
        let table = unsafe { load_table(frame) };
        let entry = table[index];
        match entry.frame(level) {
            Some(f) => match f {
                Phys::Size2Mb(_) | Phys::Size1Gb(_) => {
                    frame = f;
                    break;
                }
                _ => frame = f,
            },
            None => return None,
        }
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}

/// Create a new page table mapping
///
/// This is unsafe because if we map to an existing frame
/// we can create aliased mutable references
pub unsafe fn create_mapping(addr: &VirtAddr, entry: PageTableEntry) {
    create_mapping_inner(addr, entry);
}

fn create_mapping_inner(addr: &VirtAddr, entry: PageTableEntry) {
    let (level_4_table_frame, _) = Cr3::read();
    let mut frame: Phys = level_4_table_frame.into();

    let mut table = unsafe { load_mut_table(frame) };

    for i in 0..4 {
        let level = 3 - i;
        let index = addr.page_table_index(level);
        table = unsafe { load_mut_table(frame) };

        let entry = table[index];
        match entry.frame(level) {
            Some(f) => match f {
                Phys::Size2Mb(_) | Phys::Size1Gb(_) => {
                    break;
                }
                _ => frame = f,
            },
            None => {
                if level != 0 {
                    panic!("allocation of new page table frames not yet supported");
                }
            }
        }
    }

    table[addr.page_table_index(0)] = entry;
}

#[inline]
fn get_offset() -> VirtAddr {
    match PHYSICAL_OFFSET.wait() {
        Some(offset) => VirtAddr::new(*offset),
        None => panic!("virtual memory system is not initialized"),
    }
}

/// Load a page table from a physical frame address
///
/// This is unsafe because it transmutes the start address of the frame into a page table
/// If it doesn't actually point to a page table memory corruption could occur
#[inline]
unsafe fn load_table<'a>(frame: Phys) -> &'a PageTable {
    let virt = get_offset() + frame.start_address().as_u64();
    let table_ptr: *const PageTable = virt.as_ptr();

    &*table_ptr
}

/// Load a mutable page table from a physical frame address
///
/// This is unsafe because it transmutes the start address of the frame into a page table
/// If it doesn't actually point to a page table memory corruption could occur
#[inline]
unsafe fn load_mut_table<'a>(frame: Phys) -> &'a mut PageTable {
    let virt = get_offset() + frame.start_address().as_u64();
    let table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *table_ptr
}

#[cfg(test)]
mod tests {
    use crate::{vga_buffer::VGA_BUFFER_ADDRESS, virt_addr::VirtAddr};

    use super::{get_offset, translate_addr};

    // We know the VGA buffer is identity mapped by the bootloader
    #[test_case]
    fn translate_vga_address() {
        let addr = VirtAddr::new(VGA_BUFFER_ADDRESS);
        let phys_addr = translate_addr(&addr);

        match phys_addr {
            Some(pa) => assert_eq!(addr.as_u64(), pa.as_u64()),
            None => panic!("vga virtual address was not mapped"),
        };
    }

    // // We know that physical address 0 is mapped & uses huge pages (This could be flaky down the line)
    #[test_case]
    fn translate_address_0() {
        // Physical Address 0 is at the map offset + 0
        let addr = get_offset();
        let phys_addr = translate_addr(&addr);

        match phys_addr {
            Some(pa) => assert_eq!(pa.as_u64(), 0),
            None => panic!("physical memory was not mapped"),
        };
    }

    #[test_case]
    fn translate_missing_address() {
        let addr = VirtAddr::new(0xDEADBEEF);
        let phys_addr = translate_addr(&addr);

        match phys_addr {
            Some(pa) => panic!("0xDEADBEEF was mapped to {} unexpectedly", pa.as_u64()),
            None => (),
        };
    }

    // Add entries to page table
    // Add entry to page table (different types too) and see if we can read it back
}
