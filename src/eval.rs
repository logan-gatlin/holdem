use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{cards::*, rank::*, state::*};

/// Convert wins/ties/losses to probability of victory
fn results_to_strength(wins: usize, ties: usize, losses: usize) -> f64 {
    let (wins, ties, losses) = (wins as f64, ties as f64, losses as f64);
    (wins + ties / 2.0) / (wins + ties + losses)
}

impl DeckState {
    pub fn board_strength(&self) -> f64 {
        let this_rank = self.current_rank();
        let (wins, ties, losses) = deck_without(self.clone())
            .into_iter()
            .combinations(2)
            .map(|opponent_hand| {
                Rank::from(best_hand_in(
                    opponent_hand.into_iter().chain(self.board.into_iter()),
                ))
            })
            .fold(
                (0, 0, 0),
                |(wins, ties, losses), opponent_rank| match this_rank.cmp(&opponent_rank) {
                    std::cmp::Ordering::Greater => (wins + 1, ties, losses),
                    std::cmp::Ordering::Equal => (wins, ties + 1, losses),
                    std::cmp::Ordering::Less => (wins, ties, losses + 1),
                },
            );
        results_to_strength(wins, ties, losses)
    }

    pub fn strength_after_n_deals(&self, n: usize) -> f64 {
        if n == 0 {
            return self.board_strength();
        }
        //let (mut wins, mut ties, mut losses) = (0, 0, 0);
        let (wins, ties, losses) = deck_without(self.clone())
            .into_iter()
            .combinations(2)
            .par_bridge()
            .fold(
                || (0, 0, 0),
                |(mut wins, mut ties, mut losses), opponent_hand| {
                    //
                    for flop in deck_without(self.clone().into_iter().chain(opponent_hand.clone()))
                        .into_iter()
                        .combinations(n)
                    {
                        let this_rank =
                            Rank::from(best_hand_in(self.into_iter().chain(flop.clone())));
                        let opponent_rank = Rank::from(best_hand_in(
                            self.board
                                .into_iter()
                                .chain(flop)
                                .chain(opponent_hand.clone()),
                        ));
                        match this_rank.cmp(&opponent_rank) {
                            std::cmp::Ordering::Greater => wins += 1,
                            std::cmp::Ordering::Equal => ties += 1,
                            std::cmp::Ordering::Less => losses += 1,
                        }
                    }
                    (wins, ties, losses)
                },
            )
            .collect::<Vec<_>>()
            .into_iter()
            .fold((0, 0, 0), |(w, t, l), (w2, t2, l2)| {
                (w + w2, t + t2, l + l2)
            });

        results_to_strength(wins, ties, losses)
    }

    /*
    pub fn hand_potential(&self) -> (f64, f64) {
        const WINS: usize = 0;
        const TIES: usize = 1;
        const LOSES: usize = 2;
        let mut hp = [[0.0; 3]; 3];
        let mut hp_total = [0.0; 3];

        let this_rank = self.current_rank();
        let deck = deck_without(self.clone());
        for opponent_hand in deck.clone().into_iter().combinations(2) {
            let opponent_rank = self.rank_with_hand([opponent_hand[0], opponent_hand[1]]);
            let index = match this_rank.cmp(&opponent_rank) {
                std::cmp::Ordering::Greater => WINS,
                std::cmp::Ordering::Equal => TIES,
                std::cmp::Ordering::Less => LOSES,
            };
            hp_total[index] += 1.0;
            for next_cards in deck.clone().into_iter().combinations(2) {
                let this_rank = Rank::from(best_hand_in(
                    self.hand
                        .into_iter()
                        .chain(self.board)
                        .chain(next_cards.clone()),
                ));
                let opponent_rank = Rank::from(best_hand_in(
                    opponent_hand
                        .clone()
                        .into_iter()
                        .chain(self.board)
                        .chain(next_cards.clone()),
                ));
                match this_rank.cmp(&opponent_rank) {
                    std::cmp::Ordering::Greater => hp[index][WINS] += 1.0,
                    std::cmp::Ordering::Equal => hp[index][TIES] += 1.0,
                    std::cmp::Ordering::Less => hp[index][LOSES] += 1.0,
                }
            }
        }
        println!("TOTAL: {hp_total:?}");
        println!("WINS: {:?}", hp[WINS]);
        println!("LOSES: {:?}", hp[LOSES]);
        println!("TIES: {:?}", hp[TIES]);
        let positive_potential = (hp[LOSES][WINS] + hp[LOSES][TIES] / 2.0 + hp[TIES][WINS] / 2.0)
            / (hp_total[LOSES] + hp_total[TIES]);
        let negative_potential = (hp[WINS][LOSES] + hp[TIES][LOSES] / 2.0 + hp[WINS][TIES] / 2.0)
            / (hp_total[WINS] + hp_total[TIES]);
        (positive_potential, negative_potential)
    }
    */
}
