#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use x86_64::VirtAddr;
use core::panic::PanicInfo;
use kernel::{println, memory::active_level_4_table};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello world{}", "!");

    kernel::init();

    #[cfg(test)]
    test_main();

    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level_4_table(phys_offset) };

    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 Entry {}: {:?}", i, entry);
        }
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
