use std::env;
use std::fs::File;
use std::io::{self, Read, Stderr, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
const BUILTIN_COMMANDS: [&str; 6] = ["echo", "type", "exit", "pwd", "cd", "cat"];
//comment
fn main() {
    let system_paths = get_system_paths();
    let Stderr = io::stderr();
    Stderr.lock().write_all(b"Welcome to the shell\n").unwrap();
    loop {
        print_prompt();
        let input = get_user_input();
        if input.trim() == "exit 0" {
            break;
        }

        process_command(&input, &system_paths);
    }
}

fn get_system_paths() -> Vec<String> {
    env::var("PATH")
        .unwrap_or_else(|_| String::new()) 
        .split(':')
        .map(String::from)
        .collect()
}

fn print_prompt() {
    print!("$ ");
    io::stdout().flush().unwrap();
}

fn get_user_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

fn process_command(input: &str, directories: &[String]) {
    let args = escape_quotes(input);

    if args.is_empty() {
        return;
    }

    match args[0].as_str() {
        "exit" => return, 
        "echo" => handle_echo(&args),
        "pwd" => handle_pwd(),
        "cd" => handle_cd(&args),
        "type" => handle_type(&args, directories),
        "cat" => handle_cat(&args),
        _ => execute_external_command(&args, directories),
    }
}

fn escape_quotes(s: &str) -> Vec<String> {
    let mut s_iter = s.trim().chars().peekable();
    let mut cur_s = String::new();
    let mut ret = Vec::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    while let Some(c) = s_iter.next() {
        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
        } else if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
        } else if c == '\\' && !in_single_quote && !in_double_quote {
            let c = s_iter.next().unwrap();
            cur_s.push(c);
        } else if c == '\\' && in_double_quote {
            match s_iter.peek().unwrap() {
                '\\' | '$' | '"' => {
                    cur_s.push(s_iter.next().unwrap());
                }
                _ => cur_s.push(c),
            };
        } else if c == ' ' && !in_single_quote && !in_double_quote {
            if !cur_s.is_empty() {
                ret.push(cur_s);
                cur_s = String::new();
            }
        } else {
            cur_s.push(c);
        }
    }
    if !cur_s.is_empty() {
        ret.push(cur_s);
    }
    ret
}



fn handle_echo(args: &[String]) {
    if args.len() > 1 {
        let output = args[1..].join(" ");
        println!("{}", output);
    }
}

fn handle_pwd() {
    if let Ok(current_dir) = env::current_dir() {
        println!("{}", current_dir.display());
    } else {
        eprintln!("pwd: error retrieving current directory");
    }
}

fn handle_cd(args: &[String]) {
    if args.len() < 2 {
        eprintln!("cd: missing argument");
        return;
    }

    let target_path = if args[1] == "~" {
        env::var("HOME").unwrap_or_else(|_| String::new())
    } else {
        args[1].to_string()
    };

    if Path::new(&target_path).exists() {
        if let Err(err) = env::set_current_dir(&target_path) {
            eprintln!("cd: {}: {}", target_path, err);
        }
    } else {
        eprintln!("cd: {}: No such file or directory", target_path);
    }
}

fn handle_type(args: &[String], directories: &[String]) {
    if args.len() < 2 {
        eprintln!("type: missing operand");
        return;
    }

    let command = &args[1];
    if BUILTIN_COMMANDS.contains(&command.as_str()) && command != "cat" {
        println!("{} is a shell builtin", command);
    } else if let Some(path) = find_command_in_path(command, directories) {
        println!("{} is {}", command, path.display());
    } else {
        eprintln!("{}: not found", command);
    }
}

fn handle_cat(args: &[String]) {
    if args.len() < 2 {
        eprintln!("cat: missing file operand");
        return;
    }

    for file_path in &args[1..] {
        let mut file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => {
                eprintln!("cat: {}: No such file", file_path);
                continue;
            }
        };

        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            print!("{}", contents);
        } else {
            eprintln!("cat: error reading {}", file_path);
        }
    }
}

fn find_command_in_path(command: &str, directories: &[String]) -> Option<PathBuf> {
    directories.iter().find_map(|dir| {
        let path = Path::new(dir).join(command);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    })
}

fn execute_external_command(args: &[String], directories: &[String]) {
    if let Some(command_path) = find_command_in_path(&args[0], directories) {
        let status = Command::new(command_path.file_name().unwrap())
            .args(&args[1..])
            .status();

        if let Err(err) = status {
            eprintln!("{}: failed to execute: {}", args[0], err);
        }
    } else {
        eprintln!("{}: command not found", args[0]);
    }
}
