#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::{
    memory::{create_mapping, translate_addr},
    paging::{PageTableEntry, PageTableEntryFlags},
    println,
    virt_addr::VirtAddr,
};
use x86_64::{structures::paging::PhysFrame, PhysAddr};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello world{}", "!");

    kernel::init(boot_info);

    #[cfg(test)]
    test_main();

    let addresses = [
        kernel::vga_buffer::VGA_BUFFER_ADDRESS, // vga buffer
                                                // 0x201008,
                                                // 0x0100_0020_1a10,
                                                // boot_info.physical_memory_offset, // Physical address 0
    ];

    for &addr in &addresses {
        let virt = VirtAddr::new(addr);
        let phys = translate_addr(&virt);
        println!("{:#x} -> {:?}", virt.as_u64(), phys);
    }

    let frame = PhysFrame::from_start_address(PhysAddr::new(4096));
    let entry = PageTableEntry::new(
        frame.unwrap(),
        PageTableEntryFlags::PRESENT | PageTableEntryFlags::WRITABLE,
    );
    unsafe {
        create_mapping(&VirtAddr::new(10), entry);
    }

    for &addr in &addresses {
        let virt = VirtAddr::new(10);
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
