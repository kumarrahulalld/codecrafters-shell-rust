use std::env;
use std::io::{self, Write};
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
//updated com
fn main() {
    let directories = get_system_paths();
    let builtin_commands = vec!["echo", "type", "exit","pwd","cd","cat"];
    
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let input = get_user_input();
        if input.trim() == "exit 0" {
            break;
        }

        process_command(&input, &builtin_commands, &directories);
    }
}

fn get_system_paths() -> Vec<String> {
    env::var("PATH")
        .unwrap()
        .split(':')
        .map(|x| x.to_string())
        .collect()
}

fn get_user_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

fn process_input(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut word = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\n' {
            continue;
        }
        else if c == '\'' && !in_double_quotes {
            in_single_quotes = !in_single_quotes;
        } else if c == '"' && !in_single_quotes {
            in_double_quotes = !in_double_quotes;
        } else if c == ' ' && !in_single_quotes && !in_double_quotes {
            if !word.is_empty() {
                result.push(word.clone());
                word.clear();
            }
        } else if c == '\\' && !in_single_quotes && !in_double_quotes {
            let c = chars.next().unwrap();
            word.push(c);
        } else {
            word.push(c);
        }
    }
    if !word.is_empty() {
        result.push(word);
    }
    result
}





fn process_command(input: &str, builtin_commands: &[&str], directories: &[String]) {
    let processed_input = process_input(input);
    let args: Vec<&str> = processed_input.iter().map(|x| x.as_str()).collect();
    if args.is_empty() {
        return;
    }
    if builtin_commands.contains(&args[0])
    {
        match args[0] {
            "echo" => handle_echo(&args),
            "pwd" => println!("{}",env::current_dir().unwrap().display()),
            "type" => handle_type(&args, builtin_commands, directories),
            "cd" => handle_cd(args[1]),
            "cat" => {
                //println!("args: {:?}", args);
                let mut contents = String::new();
                for i in 1..args.len() {
                    let mut content = String::new();
                    let mut file = std::fs::File::open(args[i]).unwrap();
                    file.read_to_string(&mut content).unwrap();
                    contents.push_str(&content);
                }
                println!("{}", contents.replace("\n", ""));
            },
            _ => println!("{}: command not found", input.trim()),
        }
    }
    else if find_command_in_path(args[0], directories).is_some() {
        Command::new(args[0]).args(&args[1..]).status().expect("failed to execute process");
    }
    else {
        println!("{}: command not found", input.trim())
    }
}

fn handle_echo(args: &[&str]) {
    //println!("args: {:?}", args);
    if args.len() > 1 {
        let parsed_input = args[1..].iter().map(|x| x.replace("\n", "")).collect::<Vec<_>>();
        println!("{}", parsed_input.join(" "));
    }
}

fn handle_cd(path:&str)
{
    if path.eq("~")
    {
        let home_dir = env::var("HOME").unwrap();
        env::set_current_dir(home_dir).unwrap();
    }
    else if Path::new(path).exists()
    {
        env::set_current_dir(path).unwrap();
    }
    else {
        println!("cd: {}: No such file or directory",path);
    }
}

fn handle_type(args: &[&str], builtin_commands: &[&str], directories: &[String]) {
    if args.len() < 2 {
        println!("type: missing operand");
        return;
    }
    
    let command = args[1];
    if builtin_commands.contains(&command) {
        println!("{} is a shell builtin", command);
    } else if let Some(path) = find_command_in_path(command, directories) {
        println!("{} is {}", command, path);
    } else {
        println!("{}: not found", command);
    }
}

fn find_command_in_path(command: &str, directories: &[String]) -> Option<String> {
    for dir in directories {
        let path = format!("{}/{}", dir, command);
        if Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}
