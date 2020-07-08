use std::fs::{OpenOptions};
use std::io::{Write};
use std::error::Error;

pub fn log(message: String) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().append(true).create(true).open("log.txt")?;
    writeln!(&mut file, "{}", message)?;
    Ok(())
}
