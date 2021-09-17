use crate::println;
use crate::print;

pub fn evaluate(command: &str) {
    if let Some(stripped) = command.strip_prefix(">>> ") {
        let res = stripped.trim();
        if res != "" {
            println!();
            println!();
            println!("[ {} ]", res);
            print!(">>> ");
        }
    }
}
