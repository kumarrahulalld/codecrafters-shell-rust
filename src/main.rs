use std::env;
#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Uncomment this block to pass the first stage
    let directories:Vec<String> = env::var("PATH").unwrap().split(":").map(|x| x.to_string()).collect();
    let builin_commands: Vec<&str> = vec!["echo", "type", "exit"];
    loop {
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    //println!("input {}", input.trim());
    if input.trim() == "exit 0" {
        break;
    }
    else if input.starts_with("echo") {
        let content  = input.trim().split(" ").collect::<Vec<&str>>();
        println!("{}", content[1..content.len()].join(" "));
    }
    else if input.starts_with("type") {
        let content  = input.trim().split(" ").collect::<Vec<&str>>();
        //println!("command - type {}", content[1]);
        if builin_commands.contains(&content[1]) {
            println!("{} is a shell builtin", content[1]);
        }
        else {
            let mut found = false;
            for dir in &directories {
                let path = format!("{}/{}", dir, content[1]);
                if std::path::Path::new(&path).exists() {
                    println!("{} is {}", content[1], path);
                    found = true;
                    break;
                }
            }
            if !found {
                println!("{}: command not found", content[1]);
            }
        }
    }
    else {
        println!("{}: command not found", input.trim());
    }
    }
}
