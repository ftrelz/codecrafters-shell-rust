use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, error::Error};

enum Builtin {
    Exit,
    Echo,
    Type,
    PWD,
    CD,
    Noop,
}

impl Builtin {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "exit" => Some(Builtin::Exit),
            "echo" => Some(Builtin::Echo),
            "type" => Some(Builtin::Type),
            "pwd" => Some(Builtin::PWD),
            "cd" => Some(Builtin::CD),
            "" => Some(Builtin::Noop),
            _ => None,
        }
    }
}

fn handle_builtins(user_input: &UserInput, done: &mut bool) -> bool {
    let Some(command) = Builtin::from_str(&user_input.command) else {
        return false;
    };

    match command {
        Builtin::Exit => {
            builtin_exit(done);
        }
        Builtin::Echo => {
            builtin_echo(user_input);
        }
        Builtin::Type => {
            builtin_type(user_input);
        }
        Builtin::PWD => {
            builtin_pwd();
        }
        Builtin::CD => {
            builtin_cd(user_input);
        }
        Builtin::Noop => (),
    }
    true
}

fn builtin_exit(done: &mut bool) {
    *done = true;
}

fn builtin_echo(user_input: &UserInput) {
    println!("{}", user_input.arguments.join(" "));
}

fn builtin_type(user_input: &UserInput) {
    let Some(first_arg) = &user_input.arguments.get(0) else {
        println!(": not found");
        return;
    };

    if let Some(_) = Builtin::from_str(first_arg) {
        println!("{} is a shell builtin", first_arg);
    } else {
        if let Some(path) = env::var_os("PATH") {
            for dir in env::split_paths(&path) {
                let file_path = dir.join(&first_arg);
                if file_path.is_file() {
                    if let Ok(meta) = fs::metadata(&file_path) {
                        if meta.permissions().mode() & 0o111 != 0 {
                            println!("{} is {}", first_arg, file_path.display());
                            return;
                        }
                    }
                }
            }
        }
        println!("{first_arg}: not found");
    }
}

fn builtin_pwd() {
    // ignore an error here as the codecrafters tests don't seem to test this
    // scenario
    if let Ok(dir) = env::current_dir() {
        println!("{}", dir.display());
    }
}

fn builtin_cd(user_input: &UserInput) {
    if let Some(arg1) = user_input.arguments.get(0) {
        if let Err(e) = env::set_current_dir(Path::new(arg1))
            && e.kind() == ErrorKind::NotFound
        {
            println!(
                "{}: {}: No such file or directory",
                &user_input.command, arg1
            );
        }
    } else {
        let home_path = env::var_os("HOME").unwrap_or(OsString::from(""));
        env::set_current_dir(Path::new(
            &home_path.into_string().unwrap_or(String::from("")),
        ))
        .ok();
    }
}

struct UserInput {
    command: String,
    arguments: Vec<String>,
}

fn parse_user_input(raw_input: &str) -> UserInput {
    let mut tokens = raw_input.split_whitespace().map(String::from);
    let command = tokens.next().unwrap_or("".to_string());
    UserInput {
        command,
        arguments: tokens.collect(),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut done = false;

    while !done {
        print!("$ ");
        io::stdout().flush()?;
        let mut raw_input = String::new();

        io::stdin().read_line(&mut raw_input)?;

        let user_input = parse_user_input(&raw_input);
        if !handle_builtins(&user_input, &mut done) {
            if let Err(_) = Command::new(&user_input.command)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .args(&user_input.arguments)
                .status()
            {
                println!("{}: command not found", user_input.command);
            }
        }
    }
    Ok(())
}
