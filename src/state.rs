use crate::{cards::*, rank::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    #[default]
    BigBlind = 0,
    SmallBlind = 1,
    Button = 2,
    Cutoff = 3,
    Hijack = 4,
    Lojack = 5,
    UTG2 = 6,
    UTG1 = 7,
    UTG = 8,
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Position::BigBlind => "BB",
                Position::SmallBlind => "SB",
                Position::Button => "BTN",
                Position::Cutoff => "CO",
                Position::Hijack => "HJ",
                Position::Lojack => "LJ",
                Position::UTG2 => "UTG+2",
                Position::UTG1 => "UTG+1",
                Position::UTG => "UTG",
            }
        )
    }
}

impl Position {
    pub const ALL: [Position; 9] = [
        Self::BigBlind,
        Self::SmallBlind,
        Self::Button,
        Self::Cutoff,
        Self::Hijack,
        Self::Lojack,
        Self::UTG2,
        Self::UTG1,
        Self::UTG,
    ];

    pub fn with_n_players(n: usize) -> &'static [Position] {
        use Position::*;
        match n {
            2 => &[BigBlind, SmallBlind],
            3 => &[BigBlind, SmallBlind, UTG],
            4 => &[BigBlind, SmallBlind, Button, UTG],
            5 => &[BigBlind, SmallBlind, Button, Cutoff, UTG],
            6 => &[BigBlind, SmallBlind, Button, Cutoff, Hijack, UTG],
            7 => &[BigBlind, SmallBlind, Button, Cutoff, Hijack, Lojack, UTG],
            8 => &[
                BigBlind, SmallBlind, Button, Cutoff, Hijack, Lojack, UTG1, UTG,
            ],
            _ => &Self::ALL,
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn current_rank(&self) -> Ranking {
        self.rank_with_hand(self.hand)
    }

    pub fn rank_with_hand(&self, hand: impl IntoIterator<Item = Card>) -> Ranking {
        assert!(
            self.board != Board::PreFlop,
            "Cannot evaluate hand strength pre-flop"
        );
        Ranking::from(best_hand_in(hand.into_iter().chain(self.board)))
    }
}
