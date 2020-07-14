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

    logger::log(String::from("--------------- Starting new game ---------------"))?;
    logger::log(String::from("Cantidad de jugadores: ") + &*players.to_string())?;
    logger::log(String::from("Modo de ejecución en debug: ") + &*debug.to_string())?;

    // Create deck of cards and shuffle
    let mut deck = game::create_deck();
    game::shuffle_deck(&mut deck);

    // Create stacks and distribute the cards among them
    let cards_per_player :i32 = deck.cards.len() as i32 / players;
    let mut stacks = vec![Vec::with_capacity(cards_per_player as usize); players as usize];
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

    // Create 1 barrier per player size 2 to sync normal play order
    let mut turn_barrier = Vec::with_capacity(players as usize);
    for _ in 0..players { turn_barrier.push(Barrier::new(2)); }
    let turn_barrier = Arc::new(turn_barrier);

    // Create 1 barrier per player size 2 to sync if a player plays in the round
    let mut play_barrier = Vec::with_capacity(players as usize);
    for _ in 0..players { play_barrier.push(Barrier::new(2)); }
    let play_barrier = Arc::new(play_barrier);

    let mut handles = Vec::with_capacity(players as usize);
    for j in 1..players + 1 {
        // Clone all the tools
        let mut stacks = stacks.clone();
        let shrx_mode_c = shrx_mode.clone();
        let tx_play_c = tx_play.clone();
        let round_barrier_c = round_barrier.clone(); 
        let turn_barrier_c = turn_barrier.clone();
        let play_barrier_c = play_barrier.clone();

        // Spawn players
        let handle = thread::spawn(move || {
            // Manejar error de logger dentro del thread
            // logger::log(String::from("Player number ") + &*j.to_string() + &*String::from(" ready to play"));

            // The player j takes their stack of cards
            let my_stack = &mut stacks[(j - 1) as usize];

            // The player j takes their normal-round-turn barrier
            let my_turn = &turn_barrier_c[(j - 1) as usize];

            // The player j takes their availability-to-play barrier
            let my_play = &play_barrier_c[(j - 1) as usize];

            'game: loop {
               
                // Players wait for previous round to be over,
                // or wait for next round to be over if they can't play
                my_play.wait();

                // Players are informed of round mode
                let mode = match shrx_mode_c.lock() {
                    Ok(shrx_mode_c) => match shrx_mode_c.recv() {
                        Ok(mode) => mode,
                        Err(e) => panic!("Failed to receive from channel! {}", e),
                    },
                    Err(e) => panic!("Poisoned! {}", e)
                };
                println!("Player: {}, Mode: {}", j, mode);

                match mode {
                    "quit" => { break 'game; }
                    "normal" => {

                        // Wait for my turn to play
                        my_turn.wait();

                        let played_card = match my_stack.pop() {
                            Some(card) => card,
                            None => Card { suit: "none".to_string(), value: 0 }
                        };
                        let play :(i32, usize, Card) = (j, my_stack.len(), played_card);
                        match tx_play_c.send(play) {
                            Ok(()) => (),
                            Err(e) => panic!("Failed to send through channel! {:?}", e)
                        };

                        // Inform the host my turn is over
                        my_turn.wait();
                    }
                    "rustic" => {

                        // All players start at the same time. If a player can't play
                        // this round, the host helps advance this barrier
                        round_barrier_c.wait();

                        // They race to take a card and play it
                        let played_card = match my_stack.pop() {
                            Some(card) => card,
                            None => Card { suit: "none".to_string(), value: 0 }
                        };
                        let play :(i32, usize, Card) = (j, my_stack.len(), played_card);
                        match tx_play_c.send(play) {
                            Ok(()) => (),
                            Err(e) => panic!("Failed to send through channel! {:?}", e)
                        };
                    }
                    _ => {}
                }
            }
        });
        handles.push(handle);
    }

    let modes = ["normal", "rustic"];
    let mut empieza_normal = 0;
    let mut players_this_round = players;
    let mut plays_round = vec![true; players as usize];
    let mut scores = vec![0.0; players as usize];
    let mut done = false;

    'game: loop {

        // Random round mode
        let mode = match modes.choose(&mut rand::thread_rng()) {
            Some(mode) => mode,
            None => panic!("Error generating mode!")
        };

        // Communicate round mode and permission to play round to players
        for i in 0..players {
            if plays_round[i as usize] {
                tx_mode.send(*mode)?;
                play_barrier[i as usize].wait();
            }
        }

        match mode {
            //si normal, ordenar, el primer jugador varia -- CHEQUEAR ESTO
            // If round is normal, coordinate player turns
            &"normal" => {
                for i in 0..players {
                    let number_player = ((i+empieza_normal)%players) as usize;
                    if plays_round[number_player] {
                        turn_barrier[number_player].wait();
                        turn_barrier[number_player].wait();
                    }
                }
                empieza_normal = (empieza_normal+1)%players;
            }
            // If round is rustic and a player can't play, the host will
            // help advance the barrier so they can start playing
            &"rustic" => {
                if players_this_round != players {
                    round_barrier.wait();
                }
            }
            _ => {}
        }

        // Gather the plays from the players
        let mut plays = Vec::with_capacity(players_this_round as usize);
        for i in 0..players {
            if plays_round[i as usize] {
                match rx_play.recv() {
                    Ok(play) => plays.push(play),
                    Err(e) => panic!("Error receiving from channel! {:?}", e)
                }
            }
        }

        // read channel
        let mut card_player :Vec<(Card,i32)>  = Vec::with_capacity(players as usize);
        for (player, cards_left, played_card) in &plays {

            if plays_round[(player - 1) as usize] { // Es necesario este if????
                println!("GOT FROM {}, CARDS LEFT {}, PLAYED {} OF {}", player, cards_left, played_card.value, played_card.suit);

                card_player.push((played_card.clone(),*player));

                if *cards_left == 0 {
                    done = true;
                }
            }
        }

        // Calculate scores
        let mut valor_alto_ronda = 0;
        let mut cantidad_valor_alto_ronda = 0;
        
        for card in &card_player {
            if valor_alto_ronda < card.0.value {
                valor_alto_ronda = card.0.value    
            }
        }
        for card in &card_player {
            if valor_alto_ronda == card.0.value {
                cantidad_valor_alto_ronda = cantidad_valor_alto_ronda + 1;
            }
        }
        for card in &card_player {
            if valor_alto_ronda == card.0.value {
                scores[(card.1 - 1) as usize] += (10/cantidad_valor_alto_ronda) as f64;
            }
            
        }

        // Initially, allow all players to play in next round
        for i in 0..players {
            plays_round[i as usize] = true;
        }

        match mode {
            // If previous round was normal,
            // all players can play next round
            &"normal" => {
                players_this_round = players;
            }

            // If previous round was rustic,
            // calculate special scoring rules
            // and exclude last player from next round
            &"rustic" => {
                let first_player = &plays[0].0;
                scores[(first_player - 1) as usize] += 1.0;

                let last_player = &plays[(players_this_round - 1) as usize].0 ;
                scores[(last_player - 1) as usize] -= 5.0;

                plays_round[(last_player - 1) as usize] = false;
                players_this_round = players - 1;
            }
            _ => {}
        }

        for (i, score) in scores.iter().enumerate() {
            println!("Player {} has a score of {}", i + 1, score);
        }

        if done { break 'game; }

    }

    // Tell the players the game is over
    for i in 0..players {
        play_barrier[i as usize].wait();
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
