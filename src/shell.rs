use crate::print;
use crate::println;
use crate::vga_buffer::ScreenChar;
use crate::vga_buffer::{change_color, Color};
use alloc::vec;
use alloc::{string::String, vec::Vec};

pub fn evaluate(command: &str) {
    if let Some(stripped) = command.strip_prefix(">>> ") {
        let res = stripped.trim();
        if res != "" {
            println!();
            let parts: Vec<&str> = res.split(" ").collect();
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

fn compute_edit_distance(a: &str, b: &str) -> usize {
    let len_a = a.chars().count();
    let len_b = b.chars().count();

    if len_a < len_b {
        return compute_edit_distance(b, a);
    }

    if len_a == 0 {
        return len_b;
    } else if len_b == 0 {
        return len_a;
    }

    let len_b = len_b + 1;

    let mut pre;
    let mut tmp;
    let mut cur = vec![0; len_b];

    for i in 1..len_b {
        cur[i] = i;
    }

    for (i, ca) in a.chars().enumerate() {
        pre = cur[0];
        cur[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            tmp = cur[j + 1];
            cur[j + 1] = core::cmp::min(
                tmp + 1,
                core::cmp::min(cur[j] + 1, pre + if ca == cb { 0 } else { 1 }),
            );
            pre = tmp;
        }
    }

    cur[len_b - 1]
}

fn default(arguments: &[&str]) {
    let mut distances: Vec<(&str, usize)> = Vec::new();
    let curr = arguments[0];
    for &command in &["help", "info", "echo", "shutdown", "clear"] {
        let distance = compute_edit_distance(curr, command);
        distances.push((command, distance));
    }

    distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    println!("Error: command {} not found.", curr);
    println!("Did you mean: {}", distances[0].0);
}

fn help(_arguments: &[&str]) {
    change_color(Color::LightBlue, Color::Black);
    print!("KarxShell help menu\n\n");
    println!("[help] This message");
    println!("[info] Info about KarxOS");
    println!("[echo <arguments>] Echoes whatever arguments you pass in");
    println!("[shutdown] Shuts off the system (QEMU only)");
    println!("[clear] Clears the screen");
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
    let mut new = String::new();
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
