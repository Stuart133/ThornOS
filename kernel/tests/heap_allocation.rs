#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use alloc::{boxed::Box, vec::Vec};
use bootloader::{entry_point, BootInfo};
use kernel::allocator::{init_heap, FRAME_ALLOCATOR, HEAP_SIZE};

extern crate alloc;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    kernel::init(boot_info);
    let alloc = match FRAME_ALLOCATOR.wait() {
        Some(a) => a,
        None => panic!("boot info allocator not initialized"),
    };
    let result = init_heap(&mut *alloc.lock());
    match result {
        Ok(_) => {}
        Err(_) => panic!("init heap failed"),
    }

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info);
}

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }

    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}
