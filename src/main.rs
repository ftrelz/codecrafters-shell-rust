use std::error::Error;
#[allow(unused_imports)]
use std::io::{self, Write};

enum Builtin {
    Exit,
    Echo,
    Type,
    Noop,
}

impl Builtin {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "exit" => Some(Builtin::Exit),
            "echo" => Some(Builtin::Echo),
            "type" => Some(Builtin::Type),
            "" => Some(Builtin::Noop),
            _ => None,
        }
    }
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

    match Builtin::from_str(first_arg) {
        Some(_) => {
            println!("{} is a shell builtin", first_arg);
        }
        None => {
            println!("{}: not found", first_arg);
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
        Builtin::Noop => (),
    }
    true
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
            println!("{}: command not found", user_input.command);
        }
    }
    Ok(())
}
