#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use x86_64::{PhysAddr, structures::paging::PhysFrame};
use core::panic::PanicInfo;
use kernel::{memory::{translate_addr, create_mapping}, println, virt_addr::VirtAddr, paging::{PageTableEntry, PageTableEntryFlags}, vga_buffer::VGA_BUFFER_ADDRESS};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello world{}", "!");

    kernel::init(boot_info);

    #[cfg(test)]
    test_main();

    let zero_addr = VirtAddr::new(0);
    let entry = PageTableEntry::new(PhysFrame::from_start_address(PhysAddr::new(VGA_BUFFER_ADDRESS)).unwrap(), PageTableEntryFlags::PRESENT | PageTableEntryFlags::WRITABLE);
    unsafe { create_mapping(&zero_addr, entry) };

    let addresses = [
        VGA_BUFFER_ADDRESS, // vga buffer
        0,
    ];

    for addr in addresses {
        let virt = VirtAddr::new(addr);
        let phys = translate_addr(&virt);
        println!("{:#x} -> {:?}", virt.as_u64(), phys);
    }

    let ptr: *mut u64 = zero_addr.as_mut_ptr();
    unsafe { ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

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
