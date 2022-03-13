#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::{memory::translate_addr, println};
use x86_64::VirtAddr;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello world{}", "!");

    kernel::init(boot_info);

    #[cfg(test)]
    test_main();

    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let addresses = [
        0xb8000, // vga buffer
        0x201008,
        0x0100_0020_1a10,
        boot_info.physical_memory_offset, // Physical address 0
    ];

    for &addr in &addresses {
        let virt = VirtAddr::new(addr);
        let phys = unsafe { translate_addr(virt, phys_offset) };
        println!("{:?} -> {:?}", virt, phys);
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
