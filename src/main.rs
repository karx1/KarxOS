#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#![feature(abi_x86_interrupt)]

mod vga_buffer;
mod interrupts;
mod gdt;
use core::panic::PanicInfo;


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

fn init() {
    gdt::init_gdt();
    interrupts::init_idt();
}

#[no_mangle]
pub extern "C" fn _start() {
    println!("Hello, world");

    init();

    x86_64::instructions::interrupts::int3();

    fn stack_overflow() {
        stack_overflow();
    }

    stack_overflow();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}
