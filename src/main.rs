#![allow(non_snake_case)]
#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#![feature(abi_x86_interrupt)]

mod vga_buffer;
mod interrupts;
mod gdt;
mod shell;
use core::panic::PanicInfo;


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

fn init() {
    gdt::init_gdt();
    interrupts::init();
}

#[no_mangle]
pub extern "C" fn _start() {
    use crate::vga_buffer::{change_color, Color};

    init();
    print!("[ ");
    change_color(Color::Green, Color::Black);
    print!("OK");
    change_color(Color::White, Color::Black);
    println!(" ] Initialized GDT and interrupts");


    print!("Welcome to ");
    change_color(Color::Blue, Color::Black);
    println!("KarxOS!");
    change_color(Color::White, Color::Black);

    // First prompt, future prompts will be handled by shell::evaluate
    print!(">>> ");

    #[cfg(test)]
    test_main();

    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}
