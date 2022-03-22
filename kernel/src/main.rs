#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::{
    allocator::ALLOCATOR,
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
    let alloc = match ALLOCATOR.wait() {
        Some(a) => a,
        None => panic!("boot info allocator not initialized"),
    };

    #[cfg(test)]
    test_main();

    let zero_addr = VirtAddr::new(0xDEADBEEF);
    let frame =
        PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(VGA_BUFFER_ADDRESS)).unwrap();
    let entry = PageTableEntry::new(
        frame,
        PageTableEntryFlags::PRESENT | PageTableEntryFlags::WRITABLE,
    );

    unsafe { create_mapping(&zero_addr, entry, &mut *alloc.lock()) };

    let addresses = [0xDEADBEEF];

    for addr in addresses {
        let virt = VirtAddr::new(addr);
        let phys = translate_addr(&virt);
        println!("{:#x} -> {:?}", virt.as_u64(), phys);
    }

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
