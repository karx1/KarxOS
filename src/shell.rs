use crate::print;
use crate::println;
use crate::vga_buffer::ScreenChar;
use crate::vga_buffer::{change_color, Color};
use arrayvec::{ArrayString, ArrayVec};

pub fn evaluate(command: &str) {
    if let Some(stripped) = command.strip_prefix(">>> ") {
        let res = stripped.trim();
        if res != "" {
            println!();
            let parts: ArrayVec<&str, 80> = res.split(" ").collect();
            let selected = match parts[0] {
                "help" => help,
                "info" => info,
                "echo" => echo,
                "shutdown" => shutdown,
                "clear" => clear,
                _ => default,
            };
            selected(&parts[..]);
            print!(">>> ");
        }
    }
}

fn default(_arguments: &[&str]) {
    println!("Error: unknown command.");
}

fn help(_arguments: &[&str]) {
    change_color(Color::LightBlue, Color::Black);
    print!("KarxShell help menu\n\n");
    println!("[help] This message");
    println!("[info] Info about KarxOS");
    println!("[echo <arguments>] Echoes whatever arguments you pass in");
    println!("[shutdown] Shuts off the system (QEMU only)");
    change_color(Color::White, Color::Black);
}

fn info(_arguments: &[&str]) {
    print!("KarxOS by ");
    change_color(Color::Blue, Color::Black);
    println!("karx (karx1 on GitHub)");
    change_color(Color::White, Color::Black);
    print!("Developed for a science project in the ");
    change_color(Color::Red, Color::Black);
    println!("Rust language.");
    change_color(Color::White, Color::Black);
    println!("If you're having input problems, switch to a US keyboard layout.");
}

fn echo(arguments: &[&str]) {
    // Join the arguments back into an ArrayString
    let mut new: ArrayString<80> = ArrayString::new();
    for arg in &arguments[1..] {
        new.push_str(arg);
        new.push(' ');
    }

    println!("{}", new);
}

fn shutdown(_arguments: &[&str]) {
    use x86_64::instructions::port::Port;

    println!("KarxOS shutting down!");
    // QEMU shutdown hack
    // TODO: acpi shutdown
    let mut shutdown_port: Port<u16> = Port::new(0x604);
    unsafe {
        shutdown_port.write(0x2000);
    }
}

fn clear(_arguments: &[&str]) {
    let mut writer = crate::vga_buffer::WRITER.lock();

    for row in 0..crate::vga_buffer::BUFFER_HEIGHT {
        for col in 0..crate::vga_buffer::BUFFER_WIDTH {
            let blank = ScreenChar {
                ascii_character: b' ',
                color_code: crate::vga_buffer::ColorCode::new(
                    crate::vga_buffer::Color::White,
                    crate::vga_buffer::Color::Black,
                ),
            };

            writer.buffer.chars[row][col].write(blank);
        }
    }
}
