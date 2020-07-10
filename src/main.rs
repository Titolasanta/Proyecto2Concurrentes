extern crate rand;

use std::error::Error;
use std::thread;
use crate::game::Card;

mod validator;
mod logger;
mod game;

fn main() -> Result<(), Box<dyn Error>> {
	let (players, debug) = validator::validate_arguments()?;

    logger::log(String::from("--------------- Starting new run ---------------"))?;
    logger::log(String::from("Cantidad de jugadores: ") + &*players.to_string())?;
    logger::log(String::from("Modo de ejecución en debug: ") + &*debug.to_string())?;

    // Create deck of cards and shuffle
    let mut deck = game::create_deck();
    game::shuffle_deck(&mut deck);

    // Create stacks and distribute the cards among them
    let cards_per_player = deck.cards.len() as i32 / players;
    let mut stacks :Vec<Vec<Card>> = vec![Vec::with_capacity(cards_per_player as usize); players as usize];
    for _ in 0..cards_per_player {
        for i in 0..players as usize {
            match deck.cards.pop() {
                Some(card) => stacks[i].push(card),
                None => ()
            }
        }
    }

    let mut handles = vec![];
    for j in 1..players + 1 {
        let stacks = stacks.clone();
        let handle = thread::spawn(move || {
            // logger::log(String::from("Player number ") + &*j.to_string() + &*String::from(" ready to play"));
            // manejar error de logger dentro del thread

            // The player j takes their stack of cards
            let my_stack = &stacks[(j - 1) as usize];
            for card in my_stack {
                println!("Card {} of {}", card.value, card.suit);
            }
            // Recibir modo de juego de la ronda
            // Si es rustica, con lock o mutex se representa la carrera por colocar la carta
            // en un channel. Propongo que se comunique una tupla (j,carta)
            // Si es normal, de alguna forma hay que sincronizar a los threads para que coloquen
            // la carta en el channel segun su numero de jugador j

        });
        handles.push(handle);
    }

    // En loop del juego, decidir si es rustico o normal aleatoriamente y comunicarselo a los jugadores
    // Una vez recibidos todos los datos (potencialmente haciendo a este proceso esperar en una barrier
    // hasta que todos los jugadores pongan una carta y lleguen a la misma barrier), calcular puntos


    for handle in handles {
        match handle.join() {
            Ok(()) => (),
            Err(e) => println!("A thread panicked! Error: {:?}", e)
        }
    }
    Ok(())
}
