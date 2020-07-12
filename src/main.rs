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

    // Create a channel through which the host will communicate round mode to the players
    let (tx_mode, rx_mode) :(Sender<&str>, Receiver<&str>) = mpsc::channel();
    let shrx_mode = Arc::new(Mutex::new(rx_mode));

    // Create a channel through which players will communicate their play to the host
    let (tx_play, rx_play) :(Sender<(i32, usize, Card)>, Receiver<(i32, usize, Card)>) = mpsc::channel();

    // Create a barrier to sync players through the rounds
    let round_barrier = Arc::new(Barrier::new(players as usize));

    let mut handles = vec![];
    for j in 1..players + 1 {
        // Clone all the tools
        let mut stacks = stacks.clone();
        let shrx_mode_c = shrx_mode.clone();
        let tx_play_c = tx_play.clone();
        let round_barrier_c = round_barrier.clone();

        // Spawn players
        let handle = thread::spawn(move || {
            // Manejar error de logger dentro del thread
            // logger::log(String::from("Player number ") + &*j.to_string() + &*String::from(" ready to play"));

            // The player j takes their stack of cards
            let my_stack = &mut stacks[(j - 1) as usize];

            'game: loop {
                // Mejorar el manejo de los errores en estos match
                let mode = match shrx_mode_c.lock() {
                    Ok(shrx_mode_c) => match shrx_mode_c.recv() {
                        Ok(mode) => mode,
                        Err(_e) => panic!("Failed to receive from channel!"),
                    },
                    Err(_e) => panic!("Poisoned!")
                };

                // Creo que habria que poner una barrier aca para que todos los
                // jugadores empiecen a la vez. Si no, es injusto
                // O quizas deberia estar dentro del match, dentro de "normal" y "rustico"

                println!("Player: {}, Mode: {}", j, mode);
                match mode {
                    "quit" => {
                        break 'game;
                    }
                    "normal" => {
                        // Si es normal, de alguna forma hay que sincronizar a los threads para que coloquen
                        // la carta en el channel segun su numero de jugador j
                        // PLACEHOLDER: POR AHORA HACE LO MISMO QUE EN RUSTICO
                        let played_card = match my_stack.pop() {
                            Some(card) => card,
                            None => Card { suit: "wtf do i do".to_string(), value: 0 }
                        };
                        let play :(i32, usize, Card) = (j, my_stack.len(), played_card);
                        tx_play_c.send(play).unwrap();
                    }
                    "rustico" => {
                        // Si es rustica, simplemente es una race condition al send.
                        // Los match piden que en cada rama se devuelva algo del mismo tipo
                        let played_card = match my_stack.pop() {
                            Some(card) => card,
                            None => Card { suit: "wtf do i do".to_string(), value: 0 }
                        };
                        let play :(i32, usize, Card) = (j, my_stack.len(), played_card);
                        match tx_play_c.send(play) {
                            Ok(()) => (),
                            Err(e) => println!("No se que hacer aca {:?}", e)
                        };
                    }
                    _ => {}
                }
                round_barrier_c.wait();
            }
        });
        handles.push(handle);
    }

    // En loop del juego, decidir si es rustico o normal aleatoriamente y comunicarselo a los jugadores
    // Una vez recibidos todos los datos calcular puntos
    let modes = ["normal", "rustico"];
    let mut done = false;
    'game: loop {
        // Random round mode
        let mode = match modes.choose(&mut rand::thread_rng()) {
            Some(mode) => mode,
            None => panic!("Error!")
        };

        // Communicate round mode to players
        for _ in 0..players {
            tx_mode.send(*mode)?;
        }

        // Gather the plays from the players
        let mut plays = Vec::with_capacity(players as usize);
        for _ in 0..players {
            match rx_play.recv() {
                Ok(play) => plays.push(play),
                Err(e) => panic!("Error! {:?}", e)
            }
        }

        // Based on the plays, calculate score and determine whether game is over or not
        for (player, cards_left, played_card) in plays {
            println!("GOT FROM {}, CARDS LEFT {}, PLAYED {} OF {}", player, cards_left, played_card.value, played_card.suit);
            // calcular y guardar scores
            if cards_left == 0 {
                done = true;
            }
        }

        if done { break 'game; }

    }

    // Tell the players the game is over
    for _ in 0..players {
        tx_mode.send("quit")?;
    }

    // Wait for players to end their execution
    for handle in handles {
        match handle.join() {
            Ok(()) => (),
            Err(e) => panic!("A thread panicked! Error: {:?}", e),
        }
    }

    Ok(())
}
