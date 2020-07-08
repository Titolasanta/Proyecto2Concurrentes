extern crate rand;

use std::error::Error;

mod validator;
mod logger;
mod game;

fn main() -> Result<(), Box<dyn Error>> {
	let (players, debug) = validator::validate_arguments()?;

    logger::log(String::from("--------------- Starting new run ---------------"))?;
    logger::log(String::from("Cantidad de jugadores: ") + &*players.to_string())?;
    logger::log(String::from("Modo de ejecución en debug: ") + &*debug.to_string())?;

    let mut deck = game::create_deck();
    game::shuffle_deck(&mut deck);

    for card in deck.cards {
        println!("Card {} of {}", card.value, card.suit);
    }

    Ok(())
}
