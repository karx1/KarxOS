use crate::println;

pub fn evaluate(command: &str) {
    let res = command.trim();
    if res != "" {
        println!();
        println!();
        println!("[ {:#?} ]", res);
    }
}
