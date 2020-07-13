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

    //Create 1 barrier per player size 2 to syncronyze normal play order
     //no se como no copiarle todo el array a cada thread
    let mut barrier_order : Vec<Barrier>  = Vec::with_capacity(players as usize);

    for _ in 0..players {
        barrier_order.push(Barrier::new(2));
    }
    let barrier_order = Arc::new(barrier_order);

    //Create 1 barrier per player size 2 to syncronyze if a player plays in the round
    //no se como no copiarle todo el array a cada thread
    let mut barrier_round : Vec<Barrier>  = Vec::with_capacity(players as usize);

    for _ in 0..players {
        barrier_round.push(Barrier::new(2));
    }
    let barrier_round = Arc::new(barrier_round);


    let mut handles = vec![];
    for j in 1..players + 1 {
        // Clone all the tools
        let mut stacks = stacks.clone();
        let shrx_mode_c = shrx_mode.clone();
        let tx_play_c = tx_play.clone();
        let round_barrier_c = round_barrier.clone(); 
        //no se como no copiarle todo el array a cada thread
        let barrier_order_c = barrier_order.clone(); 
        let barrier_order_c2 = barrier_round.clone(); 

        // Spawn players
        let handle = thread::spawn(move || {
            // Manejar error de logger dentro del thread
            // logger::log(String::from("Player number ") + &*j.to_string() + &*String::from(" ready to play"));

            // The player j takes their stack of cards
            let my_stack = &mut stacks[(j - 1) as usize];

            let my_turn = &barrier_order_c[(j -1) as usize];
            
            let my_round = &barrier_order_c2[(j -1) as usize];
      
            let next_turn = &barrier_order_c[(j%players) as usize];
            
            
            'game: loop {
               
                //me toca esta ronda?
                my_round.wait();    


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

                        //es mi turno?
                        my_turn.wait();
                        let played_card = match my_stack.pop() {
                            Some(card) => card,
                            None => Card { suit: "wtf do i do".to_string(), value: 0 }
                        };
                        let play :(i32, usize, Card) = (j, my_stack.len(), played_card);
                        tx_play_c.send(play).unwrap();
                        
                        //es el turno del siguiente?
                        next_turn.wait();
                            
                    }
                    "rustico" => {
                        //empiezan todos juntos, si falta un jugador x q la anterior 
                        //fue rustico el handler se encarga del ultimo wait
                        round_barrier_c.wait();

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
                //espero que el handler me de paso 
            }
        });

        handles.push(handle);
    }

    // En loop del juego, decidir si es rustico o normal aleatoriamente y comunicarselo a los jugadores
    // Una vez recibidos todos los datos calcular puntos
    let modes = ["normal", "rustico"];
    let mut empieza_normal = 0;
    let mut players_this_round = players;
    let mut juega_ronda : Vec<bool>  = Vec::with_capacity(players as usize);
    
    for _ in 0..players {
        juega_ronda.push(true);
    }

    let mut score_player : Vec<i32>  = Vec::with_capacity(players as usize);

    for _ in 0..players {
        score_player.push(0);
    }








    let mut done = false;

    'game: loop {
        // Random round mode
        let mode = match modes.choose(&mut rand::thread_rng()) {
            Some(mode) => mode,
            None => panic!("Error!")
        };



        // Communicate round mode to players
        for i in 0..players {
            if juega_ronda[i as usize] {
                tx_mode.send(*mode)?;
                barrier_round[i as usize].wait();
            }
        }


        //si normal, ordenar, el primer jugador varia, 
        if &modes[0] == mode {
            if !(juega_ronda[empieza_normal as usize]) {
                empieza_normal+=1;

            }

            barrier_order[empieza_normal as usize].wait();

            for i in 0..players {
                if !(juega_ronda[i as usize]){
                    barrier_order[i as usize].wait();
                    if empieza_normal != ( (i+1)%players){
                        barrier_order[((i+1)%players) as usize].wait();
                    }
                }
            }
            //si el anterior al que empezo jugo
            if juega_ronda[(((empieza_normal-1)+players)%players) as usize] == true{
                barrier_order[empieza_normal as usize].wait();
            }

            empieza_normal = (empieza_normal+1)%4;
        }
            


        else {
            if players != players_this_round{
                round_barrier.wait();
            }
        }

        // Gather the plays from the players
        let mut plays = Vec::with_capacity(players_this_round as usize);

        for i in 0..players {
            if juega_ronda[i as usize] {
                match rx_play.recv() {
                    Ok(play) => plays.push(play),
                    Err(e) => panic!("Error! {:?}", e)
                }

            }
        }
        // Based on the plays, calculate score and determine whether game is over or not
        let mut card_player : Vec<(Card,i32)>  = Vec::with_capacity(players as usize);
        for (player, cards_left, played_card) in &plays {

            if juega_ronda[(player-1) as usize] {
                println!("GOT FROM {}, CARDS LEFT {}, PLAYED {} OF {}", player, cards_left, played_card.value, played_card.suit);
     

                card_player.push ((played_card.clone(),*player));


                if *cards_left == 0 {
                    done = true;
                }
            }
        }
        //update scores
        //normal
        let mut valor_alto_ronda=0;
        let mut cantidad_valor_alto_ronda = 0;
        // calcular y guardar scores
        
        for card in &card_player {
            if valor_alto_ronda < card.0.value {
                valor_alto_ronda = card.0.value    
            }
        }
        for card in &card_player {
            if valor_alto_ronda == card.0.value {
                cantidad_valor_alto_ronda = cantidad_valor_alto_ronda+1;    
            }
        }

        for card in &card_player {
             if valor_alto_ronda == card.0.value {
                score_player[(card.1-1) as usize] += 10/cantidad_valor_alto_ronda;    
            }
            
        }

        for i in 0..players {
            juega_ronda[i as usize] = true;
        }

        if &modes[1] == mode {
            let first_player = &plays[0].0;
            score_player[ (first_player-1) as usize]+=1;
            
            let last_player =&plays[ (players_this_round-1) as usize].0 ;
            score_player[(last_player-1) as usize] -= 5;

            
           juega_ronda[ (last_player-1) as usize] = false;
           players_this_round = players - 1
                   
        }else {
            
            players_this_round = players;
        }

        if done { break 'game; }

    }

    // Tell the players the game is over
    for i in 0..players {
        barrier_round[i as usize].wait();
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
