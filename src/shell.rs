use crate::println;
use crate::print;
use crate::vga_buffer::{change_color, Color};
use arrayvec::{ArrayVec, ArrayString};

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
                _ => default 
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
    let mut new: ArrayString<80> = ArrayString::new();
    for arg in &arguments[1..] {
        new.push_str(arg);
        new.push(' ');
    }

    println!("{}", new);
}
