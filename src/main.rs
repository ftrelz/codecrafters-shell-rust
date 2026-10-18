#[allow(unused_imports)]
use std::io::{self, Write};

fn builtin_exit(done: &mut bool) {
    *done = true;
}

fn handle_builtins(user_input: &str, done: &mut bool) -> bool {
    match user_input {
        "exit" => {
            builtin_exit(done);
            true
        }
        _ => false,
    }
}

fn main() -> io::Result<()> {
    let mut done = false;

    print!("$ ");
    io::stdout().flush()?;
    while !done {
        let mut raw_input = String::new();

        io::stdin().read_line(&mut raw_input)?;
        let trimmed = raw_input.trim().to_string();

        if !handle_builtins(trimmed.as_str(), &mut done) {
            println!("{trimmed}: command not found");
        }

        print!("$ ");
        io::stdout().flush()?;
    }
    Ok(())
}
