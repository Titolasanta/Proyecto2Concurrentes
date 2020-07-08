extern crate rand;
use std::fs::{OpenOptions};
use std::io::{Write};
use std::error::Error;

mod validator;
mod game;

fn main() -> Result<(), Box<dyn Error>> {
	let (players, debug) = validator::validate_arguments()?;

    println!("Cantidad de jugadores: {:?}", players);
    println!("Modo de ejecución en debug: {:?}", debug);

    let mut file = OpenOptions::new().append(true).create(true).open("log.txt")?;
    writeln!(&mut file, "Hello, world!")?;

    let mut deck = game::create_deck();
    game::shuffle_deck(&mut deck);

    for card in deck.cards {
        println!("Card {} of {}", card.value, card.suit);
    }

    Ok(())
}
