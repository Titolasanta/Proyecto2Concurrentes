extern crate rand;
use rand::seq::SliceRandom;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;

fn main()-> Result<(), Box<dyn Error>> {
	let expected_arguments = 2;

    let args: Vec<String> = env::args().collect();

    if args.len() != (expected_arguments + 1) {
    	Err("Number of players and run mode must be specified")?;
    }

    let players_amount:i32 = match args[1].parse::<i32>() {
        Ok(players_amount)  => players_amount,
        Err(e) => return Err(Box::new(e)),
    };
    if players_amount < 4 || players_amount % 2 != 0 {
        Err("Number of players must be even and at least 4")?;
    }
    let debug_mode = args[2] == String::from("debug");

    println!("Cantidad de jugadores: {:?}", players_amount);
    println!("Modo de ejecucion en debug: {:?}", debug_mode);

    let mut file = File::create("log.txt")?;
    file.write_all(b"Hello, world!")?;

    struct Card {
        suit: String,
        value: i32
    }

    struct Deck {
        cards: Vec<Card>
    }

    let mut cards: Vec<Card> = Vec::with_capacity(48);
    for suit in &["Spades", "Diamonds", "Clubs", "Hearts"] {
        for value in 1..13 {
            cards.push(Card { suit: suit.to_string(), value });
        }
    }

    let mut deck: Deck = Deck { cards: Vec::with_capacity(48) };
    let mut rng = rand::thread_rng();
    cards.shuffle(&mut rng);
    deck.cards = cards;
    for card in deck.cards {
        println!("Card {} of {}", card.value, card.suit);
    }

    Ok(())
}
