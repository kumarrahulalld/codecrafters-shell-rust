#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Uncomment this block to pass the first stage
    loop {
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
        println!("{}", content[1..content.len()].join(" "));
    }
    else if input.starts_with("type") {
        let content  = input.trim().split(" ").collect::<Vec<&str>>();
        match content[1] {
            "echo" => println!("echo is a shell builtin"),
            "type" => println!("type is a shell builtin"),
            "exit" => println!("type is a shell builtin"),
            _ => println!("{}: command not found", content[1]),
        }
    }
    else {
        println!("{}: command not found", input.trim());
    }
    }
}
