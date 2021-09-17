use crate::println;
use crate::print;

pub fn evaluate(command: &str) {
    let res = command.trim();
    if res != "" {
        println!();
        println!();
        println!("[ {:#?} ]", res);
        print!(">>> ");
    }
}
