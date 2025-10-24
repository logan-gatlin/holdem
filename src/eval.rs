use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

use crate::{cards::*, preflop::HandChart, rank::*, state::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
pub enum Recommendation {
    #[default]
    Fold = 0,
    Call = 1,
    Raise = 2,
    AllIn = 3,
}

impl std::fmt::Display for Recommendation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Recommendation::Fold => "Check / fold",
                Recommendation::Call => "Call",
                Recommendation::Raise => "Raise",
                Recommendation::AllIn => "All in",
            }
        )
    }
}

impl Recommendation {
    pub fn symbol(self) -> u8 {
        match self {
            Recommendation::Fold => 'f' as u8,
            Recommendation::Call => 'c' as u8,
            Recommendation::Raise => 'r' as u8,
            Recommendation::AllIn => 'a' as u8,
        }
    }
}

/// Convert wins/ties/losses to probability of victory
fn results_to_strength(wins: usize, ties: usize, losses: usize) -> f64 {
    let (wins, ties, losses) = (wins as f64, ties as f64, losses as f64);
    (wins + ties / 2.0) / (wins + ties + losses)
}

impl DeckState {
    pub fn strength(&self) -> f64 {
        match self.board {
            Board::PreFlop => self.preflop_strength(),
            Board::Flop(_) => self.strength_after_n_deals(2),
            Board::Turn(_) => self.strength_after_n_deals(1),
            Board::River(_) => self.board_strength(),
        }
    }

    /// Strength of the hand given only the cards on the board
    fn board_strength(&self) -> f64 {
        let opponent_range = if self.variance == Variance::Random {
            HandChart::filled_with(Recommendation::Call)
        } else {
            HandChart::opponent_expectation()
        };
        let this_rank = self.current_rank();
        let (wins, ties, losses) = deck_without(self.clone())
            .into_iter()
            .combinations(2)
            .filter(|hand| opponent_range.filter_hand([hand[0], hand[1]]))
            .map(|opponent_hand| {
                Ranking::from(best_hand_in(
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

    /// Strength of the hand after `n` deals
    fn strength_after_n_deals(&self, n: usize) -> f64 {
        if n == 0 {
            return self.board_strength();
        }
        let opponent_range = if self.variance == Variance::Random {
            HandChart::filled_with(Recommendation::Call)
        } else {
            HandChart::opponent_expectation()
        };
        //let (mut wins, mut ties, mut losses) = (0, 0, 0);
        let (wins, ties, losses) = deck_without(self.clone())
            .into_iter()
            .combinations(2)
            .filter(|hand| opponent_range.filter_hand([hand[0], hand[1]]))
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
                            Ranking::from(best_hand_in(self.into_iter().chain(flop.clone())));
                        let opponent_rank = Ranking::from(best_hand_in(
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

    // Optimized strength calculation considering only unique opening hands
    fn preflop_strength(&self) -> f64 {
        //let opponent_range = HandChart::opponent_expectation();
        let opponent_range = if self.variance == Variance::Random {
            HandChart::filled_with(Recommendation::Call)
        } else {
            HandChart::opponent_expectation()
        };
        let (wins, ties, losses) = unique_open_hands()
            .into_par_iter()
            .filter(|hand| opponent_range.filter_hand(*hand))
            .fold(
                || (0, 0, 0),
                |(mut wins, mut ties, mut losses), opponent_hand| {
                    for flop in deck_without(opponent_hand.into_iter().chain(self.hand))
                        .into_iter()
                        .combinations(3)
                    {
                        let this_rank =
                            Ranking::from(best_hand_in(flop.clone().into_iter().chain(self.hand)));
                        let opponent_rank =
                            Ranking::from(best_hand_in(flop.into_iter().chain(opponent_hand)));
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
}
