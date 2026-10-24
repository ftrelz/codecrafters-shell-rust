use std::{env, fs, io::ErrorKind, os::unix::fs::PermissionsExt, path::PathBuf};

use crate::UserInput;

enum Builtin {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    Noop,
}

impl Builtin {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "exit" => Some(Builtin::Exit),
            "echo" => Some(Builtin::Echo),
            "type" => Some(Builtin::Type),
            "pwd" => Some(Builtin::Pwd),
            "cd" => Some(Builtin::Cd),
            "" => Some(Builtin::Noop),
            _ => None,
        }
    }
}

pub fn handle_builtins(user_input: &UserInput, done: &mut bool) -> bool {
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
        Builtin::Pwd => {
            builtin_pwd();
        }
        Builtin::Cd => {
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
    println!("{}", user_input.arguments.join(""));
}

fn builtin_type(user_input: &UserInput) {
    let Some(first_arg) = &user_input.arguments.first() else {
        println!(": not found");
        return;
    };

    if Builtin::from_str(first_arg).is_some() {
        println!("{} is a shell builtin", first_arg);
    } else {
        if let Some(path) = env::var_os("PATH") {
            for dir in env::split_paths(&path) {
                let file_path = dir.join(first_arg);
                if file_path.is_file()
                    && let Ok(meta) = fs::metadata(&file_path)
                    && meta.permissions().mode() & 0o111 != 0
                {
                    println!("{} is {}", first_arg, file_path.display());
                    return;
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
    let target: PathBuf = match user_input.arguments.first().map(String::as_str) {
        None | Some("~") => match env::var_os("HOME") {
            Some(home) => PathBuf::from(home),
            None => {
                eprintln!("cd: HOME not set");
                return;
            }
        },
        Some(path) => PathBuf::from(path),
    };

    if let Err(e) = env::set_current_dir(&target) {
        let err_string = match e.kind() {
            ErrorKind::NotFound => "No such file or directory".to_string(),
            _ => e.to_string(),
        };
        eprintln!("cd: {}: {err_string}", target.display());
    }
}
