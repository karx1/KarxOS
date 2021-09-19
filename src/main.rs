#![allow(non_snake_case)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

mod gdt;
mod interrupts;
mod shell;
mod vga_buffer;
mod memory;

use core::panic::PanicInfo;
use bootloader::BootInfo;

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
pub extern "C" fn _start(boot_info: &'static BootInfo) {
    use crate::vga_buffer::{change_color, Color};
    use x86_64::{VirtAddr, structures::paging::Translate};

    init();
    print!("[ ");
    change_color(Color::Green, Color::Black);
    print!("OK");
    change_color(Color::White, Color::Black);
    println!(" ] Initialized GDT and interrupts");

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe { memory::init(phys_mem_offset) };

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);        
    }

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
