#![no_std]
#![no_main]

mod vga_buffer;
use core::panic::PanicInfo;


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() {
    println!("Hello there{}", "!");
    println!();
    println!("General Kenobi!");
    println!();
    println!("You are a bold one");


    loop {}
}
