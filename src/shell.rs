use crate::println;
use crate::print;
use crate::vga_buffer::{change_color, Color};

pub fn evaluate(command: &str) {
    if let Some(stripped) = command.strip_prefix(">>> ") {
        let res = stripped.trim();
        if res != "" {
            println!();
            let selected = match res {
                "help" => help,
                "info" => info,
                _ => default 
            };
            selected();
            print!(">>> ");
        }
    }
}

fn default() {
    println!("Error: unknown command.");
}

fn help() {
    change_color(Color::LightBlue, Color::Black);
    print!("KarxShell help menu\n\n");
    println!("[help] This message");
    change_color(Color::White, Color::Black);
}

fn info() {
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
