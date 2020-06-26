
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;

fn main()-> Result<(), Box<dyn Error>> {
	let expected_arguments = 2;

    let args: Vec<String> = env::args().collect();

    if args.len() != (expected_arguments+1) {
    	Err("Bad request")?;
    }


    let  players_amount:i32 = match args[1].parse::<i32>() {
        Ok(players_amount)  => players_amount,
        Err(e) => return Err(Box::new(e)),
    };
    let debug_mode = args[2] == "debug";

    println!("cantidad jugadores es {:?}", players_amount);
    println!("modod ejecucion en debug: {:?}", debug_mode);

    let mut file = File::create("foo.txt")?;
    file.write_all(b"Hello, world!")?;

    Ok(())
}
