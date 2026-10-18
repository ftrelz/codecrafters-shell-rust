#[allow(unused_imports)]
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let done = false;

    print!("$ ");
    io::stdout().flush()?;
    while !done {
        let mut raw_input = String::new();

        io::stdin().read_line(&mut raw_input)?;
        let trimmed = raw_input.trim().to_string();

        println!("{trimmed}: command not found");

        print!("$ ");
        io::stdout().flush()?;
    }
    Ok(())
}
