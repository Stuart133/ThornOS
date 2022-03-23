#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::boxed::Box;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::{
    allocator::FRAME_ALLOCATOR,
    memory::{create_mapping, translate_addr},
    paging::{PageTableEntry, PageTableEntryFlags},
    println,
    vga_buffer::VGA_BUFFER_ADDRESS,
    virt_addr::VirtAddr,
};
use x86_64::{
    structures::paging::{PhysFrame, Size4KiB},
    PhysAddr,
};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello world{}", "!");

    kernel::init(boot_info);

    #[cfg(test)]
    test_main();

    let x = Box::new(42);

    kernel::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    kernel::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info);
}
