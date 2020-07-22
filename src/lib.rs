pub mod game;
pub mod player;
pub mod scorer;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn player_play_queues_first_card_in_deck() {
        use std::sync::mpsc::channel;
        use std::sync::mpsc::{Sender, Receiver};
        use crate::game::Card;
        use crate::player::play;

    	let  (mut tx_play, rx_play) :(Sender<(i32, usize, Card)>, Receiver<(i32, usize, Card)>) = channel();
    	let mut stack = Vec::with_capacity(1);
    	let popped_card = Card { suit: "Spades".to_string(), value: 1 };
        stack.push(popped_card.clone());

    	play(&mut stack, &mut tx_play, 1);
        
        let recv_card;
        match rx_play.recv() {
                    Ok(play) => recv_card = play,
                    Err(e) => panic!("Error receiving from channel! {:?}", e)
                }
        assert_eq!(popped_card.value, recv_card.2.value);
    }

    #[test]
    fn create_deck_returns_a_48_card_deck() {
        use crate::game::create_deck;

        let mut cards_in_deck = 0;
        let mut deck = create_deck();

        for _i in 0..100 as usize {
            match deck.cards.pop() {
                Some(_card) => cards_in_deck += 1,
                None => ()
            }
        }
        assert_eq!(cards_in_deck, 48);
    }

    #[test]
    fn shuffled_deck_returns_a_48_card_deck() {
        use crate::game::shuffle_deck;
        use crate::game::create_deck;

        let mut cards_in_deck = 0;
        let mut deck = create_deck();
        shuffle_deck(&mut deck);

        for _i in 0..100 as usize {
            match deck.cards.pop() {
                Some(_card) => cards_in_deck += 1,
                None => ()
            }
        }
        assert_eq!(cards_in_deck, 48);
    }

    #[test]
    fn score_round_gives_10_points_to_normal_round_unique_winner() {
        use crate::game::Card;
        use crate::scorer::score_round;

        let mut plays = Vec::with_capacity(4 as usize);
        let mut scores = vec![0.0; 4 as usize];
        for i in 0..4 {
            let play :(i32, usize, Card) = (i+1, 1, Card { suit: "Spades".to_string(), value: i });
            plays.push(play)
        }
        score_round(&plays, &mut scores);
        
        assert_eq!(scores[3],10.0 );
    }

    #[test]
    fn score_round_gives_5_points_to_normal_round_two_winners() {
        use crate::scorer::score_round;
        use crate::game::Card;

        let mut plays = Vec::with_capacity(4 as usize);
        let mut scores = vec![0.0; 4 as usize];
        for i in 0..4 {
            let play :(i32, usize, Card) = (i+1, 1, Card { suit: "Spades".to_string(), value: i/2 });
            plays.push(play)
        }
        score_round(&plays, &mut scores);
        
        assert_eq!(scores[3], 5.0);
        assert_eq!(scores[2], 5.0);
    }

    #[test]
    fn score_round_gives_2_and_half_points_to_normal_round_four_winners() {
        use crate::game::Card;
        use crate::scorer::score_round;

        let mut plays = Vec::with_capacity(4 as usize);
        let mut scores = vec![0.0; 4 as usize];
        for i in 0..4 {
            let play :(i32, usize, Card) = (i+1, 1, Card { suit: "Spades".to_string(), value: 1 });
            plays.push(play)
        }
        score_round(&plays, &mut scores);
        
        assert_eq!( scores[3], 2.5);
        assert_eq!( scores[2], 2.5);
        assert_eq!( scores[1], 2.5);
        assert_eq!( scores[0], 2.5);
    }

    #[test]
    fn score_round_gives_2_and_half_points_to_normal_round_4_winner_with_48_players() {
        use crate::scorer::score_round;
        use crate::game::Card;

        let mut plays = Vec::with_capacity(48 as usize);
        let mut scores = vec![0.0; 48 as usize];
        for i in 0..48 {
            let play :(i32, usize, Card) = (i+1, 1, Card { suit: "Spades".to_string(), value: i/4+1 });
            plays.push(play)
        }
        score_round(&plays, &mut scores);
        assert_eq!( scores[47], 2.5);
        assert_eq!( scores[46], 2.5);
        assert_eq!( scores[45], 2.5);
        assert_eq!( scores[44], 2.5);
    }

    #[test]
    fn score_round_rustic_gives_1_to_first_player() {
        use crate::scorer::score_round_rustic;
        use crate::game::Card;

        let players_this_round = 4;
        let mut plays = Vec::with_capacity(4 as usize);
        let mut scores = vec![0.0; 4 as usize];
        for i in 0..4 {
            let play :(i32, usize, Card) = (i+1, 1, Card { suit: "Spades".to_string(), value: 1 });
            plays.push(play)
        }
        score_round_rustic(&plays, &mut scores, players_this_round);

        assert_eq!(scores[0], 1.0);
    }

    #[test]
    fn score_round_rustic_gives_minus_five_points_to_last_player() {
        use crate::game::Card;
        use crate::scorer::score_round_rustic;

        let players_this_round = 4;
        let mut plays = Vec::with_capacity(4 as usize);
        let mut scores = vec![0.0; 4 as usize];
        for i in 0..4 {
            let play :(i32, usize, Card) = (i+1, 1,Card { suit: "Spades".to_string(), value: 1 });
            plays.push(play)
        }
        score_round_rustic(&plays, &mut scores, players_this_round);

        assert_eq!(scores[3], -5.0);
    }
}
