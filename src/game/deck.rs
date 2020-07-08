pub mod deck {
    use crate::game::Card;

    pub struct Deck {
        pub cards: Vec<Card>
    }

    impl Deck {
        pub fn new(cards: Vec<Card>) -> Deck {
            Deck {
                cards
            }
        }
    }

}
