use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use linked_list_allocator::LockedHeap;
use spin::{Mutex, Once};
use x86_64::{
    structures::paging::{PageSize, PhysFrame, Size4KiB},
    PhysAddr,
};

use crate::{
    memory::load_active_pagetable,
    paging::{Page, PageRangeInclusive, PageTableEntry, PageTableEntryFlags},
    virt_addr::VirtAddr,
};

pub static FRAME_ALLOCATOR: Once<Mutex<BootInfoAllocator>> = Once::new();

#[global_allocator]
static GLOBAL_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

pub fn init_heap(frame_allocator: &mut impl FrameAllocator) -> Result<(), ()> {
    let table = load_active_pagetable();

    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START.try_into().unwrap()); // TODO: Make this less gross
        let heap_end = heap_start + (HEAP_SIZE - 1).try_into().unwrap();
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        PageRangeInclusive::new(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator.allocate().unwrap();
        let flags = PageTableEntryFlags::PRESENT | PageTableEntryFlags::WRITABLE;
        let entry = PageTableEntry::new(frame, flags);
        let page_result = unsafe { table.map_page(page, entry, frame_allocator) };
        match page_result {
            Ok(_) => {}
            Err(_) => return Err(()),
        };
    }

    unsafe {
        GLOBAL_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

/// Initialize the boot info allocator
///
/// This is unsafe because the caller must guarantee that the passed
/// memory map is valid. All froms marked as USABLE must actually be unused
pub unsafe fn init(memory_map: &'static MemoryMap) {
    FRAME_ALLOCATOR
        .call_once(|| Mutex::<BootInfoAllocator>::new(BootInfoAllocator::init(memory_map)));
}

pub trait FrameAllocator<S: PageSize = Size4KiB> {
    fn allocate(&mut self) -> Option<PhysFrame<S>>;
}

/// An allocator that always returns None
pub struct ZeroAllocator;

impl FrameAllocator for ZeroAllocator {
    fn allocate(&mut self) -> Option<PhysFrame> {
        None
    }
}

// TODO: Check out named existential types to store iterator and avoid recreating for every alloc
pub struct BootInfoAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl FrameAllocator for BootInfoAllocator {
    // TODO: Deallocate frames
    fn allocate(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

impl BootInfoAllocator {
    /// Create a new frame allocator
    ///
    /// This is unsafe because the caller must guarantee that the passed
    /// memory map is valid. All froms marked as USABLE must actually be unused
    unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Returns an iterator of usable frames from the memory map
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));

        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}
