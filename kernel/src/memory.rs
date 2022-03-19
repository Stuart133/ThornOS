use spin::Once;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::PhysFrame;
use x86_64::PhysAddr;

use crate::paging::{PageTable, PageTableEntry, PageTableEntryFlags, Phys};
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

    let (entry, level) = walkpte(level_4_table_frame.into(), addr);

    match entry {
        Some(pte) => match pte.frame(level) {
            Some(f) => Some(f.start_address() + u64::from(addr.page_offset())),
            None => None,
        },
        None => None,
    }

    // frame.map(|f| f.start_address() + u64::from(addr.page_offset()))
}

/// Create a new page table mapping
///
/// This is unsafe because if we map to an existing frame
/// we can create aliased mutable references
unsafe fn create_mapping(addr: VirtAddr, frame: PhysFrame, flags: PageTableEntryFlags) {
    // TODO: Replace addr with page
}

fn walkpte(table: Phys, addr: &VirtAddr) -> (Option<PageTableEntry>, usize) {
    let physical_memory_offset = get_offset();

    let mut frame: Phys = table;

    let mut entry: Option<PageTableEntry> = None;
    for i in 0..4 {
        let level = 3 - i;
        let index = addr.page_table_index(level);

        // Convert the frame into a page table reference
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        entry = Some(table[index]);
        match entry.unwrap().frame(level) {
            Some(f) => match f {
                Phys::Size2Mb(_) | Phys::Size1Gb(_) => {
                    return (entry, level);
                }
                _ => frame = f,
            },
            None => return (None, level),
        }
    }

    (entry, 0)
}
fn walk(table: Phys, addr: &VirtAddr) -> Option<Phys> {
    let physical_memory_offset = get_offset();

    let mut frame: Phys = table;

    for i in 0..4 {
        let index = addr.page_table_index(3 - i);

        // Convert the frame into a page table reference
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        let entry = table[index];
        match entry.frame(3 - i) {
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

    Some(frame)
}

fn get_offset() -> VirtAddr {
    match PHYSICAL_OFFSET.wait() {
        Some(offset) => VirtAddr::new(*offset),
        None => panic!("virtual memory system is not initialized"),
    }
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
