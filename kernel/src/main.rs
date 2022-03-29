#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::{
    allocator::{init_heap, FRAME_ALLOCATOR},
    println,
};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello world{}", "!");

    kernel::init(boot_info);

    #[cfg(test)]
    test_main();

    let alloc = match FRAME_ALLOCATOR.wait() {
        Some(a) => a,
        None => panic!("frame allocator not initialized"),
    };
    let result = init_heap(&mut *alloc.lock());
    match result {
        Ok(_) => {}
        Err(_) => panic!("init heap failed"),
    }

    let x = 34;
    println!("{:p}", &x);

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
