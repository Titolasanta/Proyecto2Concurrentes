use std::env;
use std::error::Error;

pub fn validate_arguments() -> Result<(i32,bool), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let expected_arguments = 1;

    if args.len() < (expected_arguments + 1) {
        Err("Number of players must be specified")?;
    }

    let players: i32 = match args[1].parse::<i32>() {
        Ok(players)  => players,
        Err(e) => return Err(Box::new(e)),
    };

    if players < 4 || players % 2 != 0 || players > 48 {
        Err("Number of players must be even and between 4 and 48")?;
    }

    let mut debug = false;
    if args.len() == 3 {
        debug = args[2] == String::from("debug");
    }

    std::result::Result::Ok((players, debug))
}
