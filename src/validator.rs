use std::env;
use std::error::Error;

pub fn validate_arguments() -> Result<(i32,bool), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let expected_arguments = 2;

    if args.len() != (expected_arguments + 1) {
        Err("Number of players and debug mode must be specified")?;
    }

    let players: i32 = match args[1].parse::<i32>() {
        Ok(players)  => players,
        Err(e) => return Err(Box::new(e)),
    };

    if players < 4 || players % 2 != 0 {
        Err("Number of players must be even and at least 4")?;
    }
    if players > 48 {
        Err("Can t have more than 48 players!")?;
    }


    let debug = args[2] == String::from("debug");
    std::result::Result::Ok((players, debug))
}
