use std::error::Error;
#[allow(unused_imports)]
use std::io::{self, Write};

fn builtin_exit(done: &mut bool) {
    *done = true;
}

fn builtin_echo(user_input: &UserInput) {
    println!("{}", user_input.arguments.join(" "));
}

fn handle_builtins(user_input: &UserInput, done: &mut bool) -> bool {
    match user_input.command.as_str() {
        "exit" => {
            builtin_exit(done);
            true
        }
        "echo" => {
            builtin_echo(user_input);
            true
        }
        _ => false,
    }
}

struct UserInput {
    command: String,
    arguments: Vec<String>,
}

fn parse_user_input(raw_input: &str) -> Option<UserInput> {
    let mut tokens = raw_input.split_whitespace().map(String::from);
    let command = tokens.next()?;
    Some(UserInput {
        command,
        arguments: tokens.collect(),
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut done = false;

    while !done {
        print!("$ ");
        io::stdout().flush()?;
        let mut raw_input = String::new();

        io::stdin().read_line(&mut raw_input)?;

        let user_input = parse_user_input(&raw_input).ok_or("")?;
        if !handle_builtins(&user_input, &mut done) {
            println!("{}: command not found", user_input.command);
        }
    }
    Ok(())
}
