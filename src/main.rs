mod builtins;
mod parser;
mod tokenizer;

use std::error::Error;
use std::io::ErrorKind;
use std::io::{self, Write};
use std::process::{Command, Stdio};

use crate::builtins::handle_builtins;
use crate::parser::parse_input;

#[derive(Debug, PartialEq)]
struct UserInput {
    command: String,
    arguments: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut done = false;

    while !done {
        print!("$ ");
        io::stdout().flush()?;
        let mut raw_input = String::new();

        io::stdin().read_line(&mut raw_input)?;
        let trimmed = raw_input.trim_end();

        let user_input = parse_input(trimmed);

        if !handle_builtins(&user_input, &mut done)
            && let Err(e) = Command::new(&user_input.command)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .args(&user_input.arguments)
                .status()
        {
            let err_string = match e.kind() {
                ErrorKind::NotFound => "command not found".to_string(),
                _ => e.to_string(),
            };
            println!("{}: {}", user_input.command, err_string);
        }
    }
    Ok(())
}
