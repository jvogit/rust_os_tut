#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os_tut::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_os_tut::{
    allocator::{self, BootInfoFrameAllocator},
    keyboard::{self, ScancodeStream},
    memory, println,
    task::{executor::Executor, simple_executor::SimpleExecutor, Task},
};
use x86_64::VirtAddr;

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os_tut::test_panic_handler(info)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    rust_os_tut::hlt_loop();
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello, world!");

    rust_os_tut::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization to not fail!");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

async fn example_42() -> u32 {
    42
}

async fn example_task() {
    let val = example_42().await;

    println!("async val {}", val);
}

#[test_case]
fn test_trivial_assertion() {
    assert_eq!(1, 1);
}
