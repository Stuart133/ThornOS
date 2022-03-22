use spin::Once;
use x86_64::registers::control::Cr3;
use x86_64::PhysAddr;

use crate::allocator::FrameAllocator;
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

/// Create a new page table mapping using allocator to allocate new page table frames
/// as required
///
/// This is unsafe because if we map to an existing frame
/// we can create aliased mutable references
pub unsafe fn create_mapping<T: FrameAllocator>(
    addr: &VirtAddr,
    entry: PageTableEntry,
    allocator: &mut T,
) {
    create_mapping_inner(addr, entry, allocator);
}

#[inline]
fn create_mapping_inner<T: FrameAllocator>(addr: &VirtAddr, entry: PageTableEntry, allocator: &mut T) {
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
                    // Set the table entry here so we can index the correct virtual address PTE level
                    table[addr.page_table_index(level)] = entry;
                    return;
                }
                _ => frame = f,
            },
            None => {
                if level != 0 {
                    let new_frame = allocator.allocate();
                    match new_frame {
                        Some(f) => {
                            // TODO: Ensure memory is cleared
                            let entry = PageTableEntry::new(
                                f,
                                PageTableEntryFlags::PRESENT | PageTableEntryFlags::WRITABLE,
                            );
                            table[index] = entry;
                            frame = Phys::Size4Kb(f);
                        }
                        None => panic!(
                            "allocation of new frame for page table level {} failed",
                            level
                        ),
                    }
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
    use x86_64::{
        structures::paging::{PhysFrame, Size4KiB},
        PhysAddr,
    };

    use crate::{
        allocator::{ZeroAllocator, FRAME_ALLOCATOR},
        paging::{PageTableEntry, PageTableEntryFlags},
        vga_buffer::VGA_BUFFER_ADDRESS,
        virt_addr::VirtAddr,
    };

    use super::{create_mapping, get_offset, translate_addr};

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

    // We know that physical address 0 is mapped & uses huge pages (This could be flaky down the line)
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

    #[test_case]
    fn add_valid_entry() {
        let addr = VirtAddr::new(5);
        let frame = PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(4096)).unwrap();
        let entry = PageTableEntry::new(frame, PageTableEntryFlags::PRESENT);

        unsafe { create_mapping(&addr, entry, &mut ZeroAllocator {}) };

        let phys_addr = translate_addr(&addr);

        match phys_addr {
            Some(pa) => assert_eq!(pa.as_u64(), 4101),
            None => panic!("new page was not mapped to correct physical frame"),
        }
    }

    // The physical memory is mapped into huge pages - Remap Offset -> 0 to ensure huge pages
    // can be mapped into correctly
    #[test_case]
    fn add_valid_huge_entry() {
        let addr = get_offset();
        let frame = PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(0)).unwrap();
        let entry = PageTableEntry::new(frame, PageTableEntryFlags::PRESENT);

        unsafe { create_mapping(&addr, entry, &mut ZeroAllocator {}) };

        let phys_addr = translate_addr(&addr);

        match phys_addr {
            Some(pa) => assert_eq!(pa.as_u64(), 0),
            None => panic!("new page was not mapped to correct physical frame"),
        }
    }

    #[test_case]
    fn add_allocation_entry() {
        let addr = VirtAddr::new(0xDEADBEEF);
        let frame = PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(4096)).unwrap();
        let entry = PageTableEntry::new(frame, PageTableEntryFlags::PRESENT);

        let alloc = match FRAME_ALLOCATOR.wait() {
            Some(a) => a,
            None => panic!("boot info allocator not initialized"),
        };

        unsafe { create_mapping(&addr, entry, &mut *alloc.lock()) };

        let phys_addr = translate_addr(&addr);

        match phys_addr {
            Some(pa) => assert_eq!(pa.as_u64(), addr.page_offset().as_u64() + 4096),
            None => panic!("new page was not mapped to correct physical frame"),
        }
    }
}
