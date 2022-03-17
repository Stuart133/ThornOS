#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::{memory::translate_addr, println, virt_addr::VirtAddr};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello world{}", "!");

    kernel::init(boot_info);

    #[cfg(test)]
    test_main();

    println!("{}", boot_info.physical_memory_offset);

    let addresses = [
        kernel::vga_buffer::VGA_BUFFER_ADDRESS, // vga buffer
        // 0x201008,
        // 0x0100_0020_1a10,
        boot_info.physical_memory_offset, // Physical address 0
    ];

    for &addr in &addresses {
        let virt = VirtAddr::new(addr);
        let phys = translate_addr(&virt);
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
