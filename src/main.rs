#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Uncomment this block to pass the first stage
    while true {
        
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    if input.trim() == "exit 0" {
        break;
    }
    else if input.starts_with("echo") {
        let content  = input.trim().split(" ").collect::<Vec<&str>>();
        println!("{}", content[1]);
    }
    println!("{}: command not found", input.trim());
    }
}
