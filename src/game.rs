extern crate rand;
use rand::seq::SliceRandom;

mod card;
mod deck;

pub use crate::game::card::card::Card;
pub use crate::game::deck::deck::Deck;

pub fn create_deck() -> Deck {
    let mut cards: Vec<Card> = Vec::with_capacity(48);
    for suit in &["Spades", "Diamonds", "Clubs", "Hearts"] {
        for value in 1..13 {
            cards.push(Card { suit: suit.to_string(), value });
        }
    }
    Deck::new(cards)
}

pub fn shuffle_deck(deck: &mut Deck) {
    let mut rng = rand::thread_rng();
    deck.cards.shuffle(&mut rng);
}
