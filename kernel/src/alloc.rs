use x86_64::structures::paging::{PageSize, PhysFrame, Size4KiB};

pub trait Allocator<S: PageSize = Size4KiB> {
    fn allocate() -> Option<PhysFrame<S>>;
}

/// An allocator that always returns None
pub struct ZeroAllocator {}

impl Allocator for ZeroAllocator {
    fn allocate() -> Option<PhysFrame> {
        None
    }
}
