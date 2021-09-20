#![allow(non_snake_case)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

mod allocator;
mod gdt;
mod interrupts;
mod memory;
mod shell;
mod vga_buffer;

use bootloader::BootInfo;
use core::panic::PanicInfo;

extern crate alloc;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

fn init() {
    gdt::init_gdt();
    interrupts::init();
}

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) {
    use crate::vga_buffer::{change_color, Color};
    use memory::BootInfoFrameAllocator;
    use x86_64::VirtAddr;

    init();
    print!("[ ");
    change_color(Color::Green, Color::Black);
    print!("OK");
    change_color(Color::White, Color::Black);
    println!(" ] Initialized GDT and interrupts");

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    print!("[ ");
    change_color(Color::Green, Color::Black);
    print!("OK");
    change_color(Color::White, Color::Black);
    println!(" ] Initialized Mapper and Frame allocator");

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");
    print!("[ ");
    change_color(Color::Green, Color::Black);
    print!("OK");
    change_color(Color::White, Color::Black);
    println!(" ] Initialized heap");

    println!();
    print!("Welcome to ");
    change_color(Color::Blue, Color::Black);
    println!("KarxOS!");
    change_color(Color::White, Color::Black);

    // First prompt, future prompts will be handled by shell::evaluate
    print!(">>> ");

    #[cfg(test)]
    test_main();

    loop {
        // Halt CPU so that usage isn't 100% all the time
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
