use crate::game::Card;
use std::sync::mpsc::{Sender};

pub fn play(my_stack: &mut Vec<Card>, tx_play_c: &mut Sender<(i32,usize,Card)>, player_number: i32) -> () {

	let played_card = match my_stack.pop() {
	    Some(card) => card,
	    None => Card { suit: "None".to_string(), value: 0 }
	};
	let play :(i32, usize, Card) = (player_number, my_stack.len(), played_card);
	match tx_play_c.send(play) {
	    Ok(()) => (),
	    Err(e) => panic!("Failed to send through channel! {:?}", e)
	};
}
