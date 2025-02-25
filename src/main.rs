use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

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

    if let Some((command_args, stdout_file, stderr_file)) = handle_redirection(&args) {
        match command_args[0].as_str() {
            "exit" => return,
            "echo" => handle_echo(&command_args, stdout_file, stderr_file),
            "pwd" => handle_pwd(stdout_file, stderr_file),
            "cd" => handle_cd(&command_args),
            "type" => handle_type(&command_args, directories),
            "cat" => handle_cat(&command_args, stdout_file, stderr_file),
            _ => execute_external_command(&command_args, directories, stdout_file, stderr_file),
        }
    } else {
        match args[0].as_str() {
            "exit" => return,
            "echo" => handle_echo(&args, None, None),
            "pwd" => handle_pwd(None, None),
            "cd" => handle_cd(&args),
            "type" => handle_type(&args, directories),
            "cat" => handle_cat(&args, None, None),
            _ => execute_external_command(&args, directories, None, None),
        }
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

fn handle_echo(args: &[String], stdout_file: Option<String>, stderr_file: Option<String>) {
    if args.len() > 1 {
        let mut output = args[1..].join(" ");
        
        if args[1] == "-n" {
            output = args[2..].join(" "); 
        }

        if let Some(file) = stdout_file {
            let mut file = File::create(file).unwrap();
            writeln!(file, "{}", output).unwrap();
        }
        else if let Some(file) = stderr_file {
            let mut file = File::create(file).unwrap();
            write!(file, "{}", output).unwrap(); // Write to stderr file
        }
         else {
            println!("{}", output); // Output without extra newline
        }
    }
}

fn handle_pwd(stdout_file: Option<String>, stderr_file: Option<String>) {
    if let Ok(current_dir) = env::current_dir() {
        let output = current_dir.display().to_string();
        if let Some(file) = stdout_file {
            let mut file = File::create(file).unwrap();
            writeln!(file, "{}", output).unwrap();
        } else {
            println!("{}", output); 
        }

        if let Some(file) = stderr_file {
            let mut file = File::create(file).unwrap();
            writeln!(file, "{}", output).unwrap(); // Write to stderr file
        }
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

fn handle_cat(args: &[String], stdout_file: Option<String>, stderr_file: Option<String>) {
    if args.len() < 2 {
        eprintln!("cat: missing file operand");
        return;
    }
    let mut contents = String::new();
    for file_path in &args[1..] {
        let mut file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => {
                eprintln!("cat: {}: No such file or directory", file_path);
                continue;
            }
        };
        let mut con = String::new();
        if file.read_to_string(&mut con).is_ok() {
            while con.ends_with("\n") {
                con.pop();
            }
            contents.push_str(&con);
        } else {
            eprintln!("cat: error reading {}", file_path);
        }
    }
    if let Some(ref file) = stdout_file {
        let mut output_file = File::create(file).unwrap();
        writeln!(output_file, "{}", contents).unwrap();
    } else {
        println!("{}", contents);
    }

    if let Some(ref file) = stderr_file {
        let mut output_file = File::create(file).unwrap();
        writeln!(output_file, "{}", contents).unwrap(); // Write to stderr file
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

fn execute_external_command(args: &[String], directories: &[String], stdout_file: Option<String>, stderr_file: Option<String>) {
    if let Some(command_path) = find_command_in_path(&args[0], directories) {
        let output = Command::new(command_path.file_name().unwrap())
            .args(&args[1..])
            .output();

        match output {
            Ok(output) => {
                let result = String::from_utf8_lossy(&output.stdout);
                if let Some(file) = stdout_file {
                    let mut file = File::create(file).unwrap();
                    writeln!(file, "{}", result).unwrap();
                } else {
                    print!("{}", result);  // Output to stdout without extra newline
                }

                let err_result = String::from_utf8_lossy(&output.stderr);
                if let Some(file) = stderr_file {
                    let mut file = File::create(file).unwrap();
                    writeln!(file, "{}", err_result).unwrap(); // Write to stderr file
                } else {
                    eprint!("{}", err_result);  // Output to stderr without extra newline
                }
            }
            Err(err) => eprintln!("{}: failed to execute: {}", args[0], err),
        }
    } else {
        eprintln!("{}: command not found", args[0]);
    }
}

// Function to handle the redirection operator
fn handle_redirection(args: &[String]) -> Option<(Vec<String>, Option<String>, Option<String>)> {
    let mut new_args = args.to_vec();
    let mut stderr_file = None;
    let mut stdout_file = None;

    // Check for either ">" or "1>" in the args (stdout redirection)
    if let Some(redirect_index) = args.iter().position(|x| x == ">" || x == "1>") {
        let filename = args.get(redirect_index + 1).cloned(); // Get the filename after the operator
        
        if let Some(filename) = filename {
            // Remove the redirection operator and filename from the arguments
            new_args.remove(redirect_index); // Remove the ">" or "1>"
            new_args.remove(redirect_index); // Remove the filename
            stdout_file = Some(filename);
        }
    }

    // Check for "2>" in the args (stderr redirection)
    if let Some(redirect_index) = args.iter().position(|x| x == "2>") {
        let filename = args.get(redirect_index + 1).cloned(); // Get the filename after the operator
        
        if let Some(filename) = filename {
            // Remove the redirection operator and filename from the arguments
            new_args.remove(redirect_index); // Remove the "2>"
            new_args.remove(redirect_index); // Remove the filename
            stderr_file = Some(filename);
        }
    }

    // Return modified arguments along with the redirection targets (stdout and stderr)
    Some((new_args, stdout_file, stderr_file))
}
