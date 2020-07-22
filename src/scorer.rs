      

use crate::game::Card;
pub fn score_round(plays:&Vec<(i32,usize,Card)>,scores: &mut Vec<f64>)->() {

        let mut highest_score_round = 0;
        let mut number_of_highest_score_round = 1;
        
        for (_player, _cards_left, played_card) in plays {
            if highest_score_round < played_card.value {
                number_of_highest_score_round = 1;
                highest_score_round = played_card.value;
            }else if highest_score_round == played_card.value {
                number_of_highest_score_round = number_of_highest_score_round + 1;
            }
        }

        for (player, _cards_left, played_card) in plays {
            if highest_score_round == played_card.value {
                scores[(player - 1) as usize] += 10.0 / number_of_highest_score_round as f64;
            }
        }    
}


pub fn score_round_rustic(plays:&Vec<(i32,usize,Card)>,scores: &mut Vec<f64>,players_this_round:i32)->() {
        let first_player = &plays[0].0;
        scores[(first_player - 1) as usize] += 1.0;

        let last_player = &plays[(players_this_round - 1) as usize].0 ;
        scores[(last_player - 1) as usize] -= 5.0;
}