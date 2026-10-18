#[allow(unused_imports)]
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let done = false;

    let mut raw_input = String::new();
    let mut trimmed;
    print!("$ ");
    io::stdout().flush()?;
    while !done {
        io::stdin().read_line(&mut raw_input)?;
        trimmed = raw_input.trim().to_string();

        println!("{trimmed}: command not found");

        print!("$ ");
        io::stdout().flush()?;
    }
    Ok(())
}
