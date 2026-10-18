#[allow(unused_imports)]
use std::io::{self, Write};

fn main() -> io::Result<()> {
    print!("$ ");
    io::stdout().flush()?;

    let mut raw_input = String::new();
    io::stdin().read_line(&mut raw_input)?;

    println!("");
    println!("{raw_input}: command not found");
    Ok(())
}
