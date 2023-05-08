#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

use lazy_static::lazy_static;
use rust_os_tut::exit_qemu;
use rust_os_tut::gdt;
use rust_os_tut::serial_print;
use rust_os_tut::serial_println;
use rust_os_tut::QemuExitCode;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    // init gdt
    rust_os_tut::gdt::init();
    // init test idt
    init_test_idt();

    serial_print!("stack_overflow::stack_overflow...\t");
    stack_overflow(); // stack overflow should cause doublefault handler to be called
    panic!("[expected execution to not continue]");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os_tut::test_panic_handler(info)
}

#[warn(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read(); // prevent tail recursion optimization
}

lazy_static! {
    pub static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(mock_double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_STACK_TABLE_INDEX);
        }

        idt
    };
}

extern "x86-interrupt" fn mock_double_fault_handler(
    _stack_info: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

fn init_test_idt() {
    TEST_IDT.load();
}
