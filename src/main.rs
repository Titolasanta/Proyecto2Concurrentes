extern crate rand;
use rand::seq::SliceRandom;

use std::error::Error;
use std::thread;
use std::sync::{mpsc, Mutex, Arc, Barrier};
use std::sync::mpsc::{Sender, Receiver};
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

    let (tx_mode, rx_mode) :(Sender<&str>, Receiver<&str>) = mpsc::channel();
    let shrx_mode = Arc::new(Mutex::new(rx_mode));

    let (tx_play, rx_play) :(Sender<(i32, usize, Card)>, Receiver<(i32, usize, Card)>) = mpsc::channel();

    let round_barrier = Arc::new(Barrier::new(players as usize));

    let mut handles = vec![];
    for j in 1..players + 1 {
        let mut stacks = stacks.clone();
        let shrx_mode_c = shrx_mode.clone();
        let tx_play_c = tx_play.clone();
        let round_barrier_c = round_barrier.clone();
        let handle = thread::spawn(move || {
            // logger::log(String::from("Player number ") + &*j.to_string() + &*String::from(" ready to play"));
            // manejar error de logger dentro del thread

            // The player j takes their stack of cards
            let my_stack = &mut stacks[(j - 1) as usize];

            loop {
                // Mejorar el manejo de los errores en estos match
                let mode = match shrx_mode_c.lock() {
                    Ok(shrx_mode_c) => match shrx_mode_c.recv() {
                        Ok(mode) => mode,
                        Err(_e) => panic!("Failed to receive from channel!"),
                    },
                    Err(_e) => panic!("Poisoned!")
                };

                println!("PLAYER: {}, Mode: {}", j, mode);
                match mode {
                    "quit" => {
                        break;
                    }
                    "normal" => {
                        // Si es normal, de alguna forma hay que sincronizar a los threads para que coloquen
                        // la carta en el channel segun su numero de jugador j
                        let played_card = match my_stack.pop() {
                            Some(card) => card,
                            None => Card { suit: "wtf do i do".to_string(), value: 0 }
                        };
                        let play :(i32, usize, Card) = (j, my_stack.len(), played_card);
                        tx_play_c.send(play).unwrap();
                    }
                    "rustico" => {
                        // Si es rustica, con lock o mutex se representa la carrera por colocar la carta
                        // en un channel. Propongo que se comunique una tupla (j,carta)
                        let played_card = match my_stack.pop() {
                            Some(card) => card,
                            None => Card { suit: "wtf do i do".to_string(), value: 0 }
                        };
                        let play :(i32, usize, Card) = (j, my_stack.len(), played_card);
                        tx_play_c.send(play).unwrap();
                    }
                    _ => {}
                }
                round_barrier_c.wait();
            }
        });
        handles.push(handle);
    }

    // En loop del juego, decidir si es rustico o normal aleatoriamente y comunicarselo a los jugadores
    // Una vez recibidos todos los datos (potencialmente haciendo a este proceso esperar en una barrier
    // hasta que todos los jugadores pongan una carta y lleguen a la misma barrier), calcular puntos
    let modes = ["normal", "rustico"];
    let mut done = false;
    'game: loop {
        let mode = match modes.choose(&mut rand::thread_rng()) {
            Some(mode) => mode,
            None => panic!("Error!")
        };

        for _ in 0..players {
            tx_mode.send(*mode)?;
        }

        let mut plays = Vec::with_capacity(players as usize);
        for _ in 0..players {
            plays.push(rx_play.recv().unwrap());
        }

        for (player, cards_left, played_card) in plays {
            println!("GOT FROM {}, CARDS LEFT {}, PLAYED {} OF {}", player, cards_left, played_card.suit, played_card.value);
            // calcular y guardar scores
            if cards_left == 0 {
                done = true;
            }
        }

        if done { break 'game; }

    }

    for _ in 0..players {
        tx_mode.send("quit")?;
    }

    for handle in handles {
        match handle.join() {
            Ok(()) => (),
            Err(e) => println!("A thread panicked! Error: {:?}", e),
        }
    }

    Ok(())
}
