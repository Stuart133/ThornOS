use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{PageSize, PhysFrame, Size4KiB},
    PhysAddr,
};

pub trait Allocator<S: PageSize = Size4KiB> {
    fn allocate(&mut self) -> Option<PhysFrame<S>>;
}

/// An allocator that always returns None
pub struct ZeroAllocator {}

impl Allocator for ZeroAllocator {
    fn allocate(&mut self) -> Option<PhysFrame> {
        None
    }
}

// TODO: Check out named existential types to store iterator and avoid recreating for every alloc
pub struct BootInfoAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl Allocator for BootInfoAllocator {
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
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
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
