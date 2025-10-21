use crate::{cards::*, rank::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Board {
    PreFlop,
    Flop([Card; 3]),
    Turn([Card; 4]),
    River([Card; 5]),
}

impl IntoIterator for Board {
    type Item = Card;
    type IntoIter = <Vec<Card> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let i = match self {
            Board::Flop(c) => c.to_vec(),
            Board::Turn(c) => c.to_vec(),
            Board::River(c) => c.to_vec(),
            Board::PreFlop => vec![],
        }
        .into_iter();
        i
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeckState {
    /// Cards currently on the board
    pub board: Board,
    /// Cards currently in my hand
    pub hand: [Card; 2],
}

impl IntoIterator for DeckState {
    type Item = Card;

    type IntoIter = <Vec<Card> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.board
            .into_iter()
            .chain(self.hand)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl DeckState {
    pub fn current_rank(&self) -> Rank {
        self.rank_with_hand(self.hand)
    }

    pub fn rank_with_hand(&self, hand: impl IntoIterator<Item = Card>) -> Rank {
        assert!(
            self.board != Board::PreFlop,
            "Cannot evaluate hand strength pre-flop"
        );
        Rank::from(best_hand_in(hand.into_iter().chain(self.board)))
    }
}
