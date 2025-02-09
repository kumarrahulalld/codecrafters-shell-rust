use std::env;
use std::fs::File;
use std::io::{self, Write, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
//comment
const BUILTIN_COMMANDS: [&str; 6] = ["echo", "type", "exit", "pwd", "cd", "cat"];

fn main() {
    let system_paths = get_system_paths();

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
        .unwrap_or_else(|_| String::new())  // Return empty string if "PATH" is not set
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
    let args = parse_input(input);

    if args.is_empty() {
        return;
    }

    match args[0].as_str() {
        "exit" => return,  // Exit the loop in main()
        "echo" => handle_echo(&args),
        "pwd" => handle_pwd(),
        "cd" => handle_cd(&args),
        "type" => handle_type(&args, directories),
        "cat" => handle_cat(&args),
        _ => execute_external_command(&args, directories),
    }
}

fn parse_input(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut word = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;

    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // Skip newlines
            '\n' => continue,

            // Toggle single quotes when outside of double quotes
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
            }

            // Toggle double quotes when outside of single quotes
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
            }

            // Handle space only when we're not inside any quotes
            ' ' if !in_single_quotes && !in_double_quotes => {
                if !word.is_empty() {
                    result.push(word.clone());
                    word.clear();
                }
            }

            // Handle backslash escaping characters
            '\\' if in_double_quotes => {
                // Inside double quotes, preserve escape sequences
                if let Some(next_char) = chars.next() {
                    match next_char {
                        '"' => word.push('"'),  // Escape double quote
                        '\\' => word.push('\\'), // Escape backslash
                        _ => word.push(next_char), // Preserve other characters
                    }
                }
            }
            '\\' if !in_single_quotes && !in_double_quotes => {
                // Outside quotes, handle escape sequence (e.g., \50)
                if let Some(next_char) = chars.next() {
                    if next_char.is_digit(10) {
                        // It's a number, treat as an octal escape (e.g., \50 is octal for 8)
                        let mut escape_sequence = next_char.to_string();
                        let mut count = 0;

                        // Capture more digits for octal escape sequence
                        while let Some(&digit) = chars.peek() {
                            if digit.is_digit(10) {
                                escape_sequence.push(digit);
                                chars.next(); // consume the digit
                                count += 1;
                            } else {
                                break;
                            }
                        }

                        if !escape_sequence.is_empty() {
                            // Parse the escape sequence as an octal value
                            if let Ok(value) = u8::from_str_radix(&escape_sequence, 8) {
                                word.push(value as char);
                            }
                        }
                    } else {
                        word.push(next_char); // Just push the character as is
                    }
                }
            }

            // Otherwise, just add the current character to the word
            _ => word.push(c),
        }
    }

    // Add the last word if it's not empty
    if !word.is_empty() {
        result.push(word);
    }

    result
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
