#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank {
    HighCard = 0,
    OnePair = 1,
    TwoPairs = 2,
    ThreeKind = 3,
    Straight = 4,
    Flush = 5,
    FullHouse = 6,
    FourKind = 7,
    StraightFlush = 8,
    RoyalFlush = 9,
}

impl std::fmt::Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Rank::HighCard => "High card",
                Rank::OnePair => "Pair",
                Rank::TwoPairs => "Two pairs",
                Rank::ThreeKind => "Three of a kind",
                Rank::Straight => "Straight",
                Rank::Flush => "Flush",
                Rank::FullHouse => "Full house",
                Rank::FourKind => "Four of a kind",
                Rank::StraightFlush => "Straight flush",
                Rank::RoyalFlush => "Royal flush",
            }
        )
    }
}

use itertools::Itertools;

use crate::cards::*;

impl From<Hand> for Rank {
    fn from(value: Hand) -> Self {
        Self::from(&value)
    }
}

impl From<&Hand> for Rank {
    fn from(hand: &Hand) -> Self {
        use Face::*;
        let mut face_values = hand.face_values();
        face_values.sort();
        // Check for flush and straight hands
        let is_flush = hand.suites_iter().all(|c| c == hand.0[0].1);
        let is_high_straight = face_values == [Ace, Ten, Jack, Queen, King].map(|f| f as usize);
        let is_straight = face_values
            == [
                face_values[0],
                face_values[0] + 1,
                face_values[0] + 2,
                face_values[0] + 3,
                face_values[0] + 4,
            ]
            || is_high_straight;
        match (is_straight, is_flush) {
            (true, true) if is_high_straight => return Rank::RoyalFlush,
            (true, true) => return Rank::StraightFlush,
            (true, false) => return Rank::Straight,
            (false, true) => return Rank::Flush,
            _ => (),
        };
        // Check for full house and N pairs
        let mut face_value_counts = [0; 14];
        face_values.iter().for_each(|f| face_value_counts[*f] += 1);
        let mut max = 0;
        let mut two_kind_count = 0; // Used to detect full house and two pair
        for count in face_value_counts {
            if max < count {
                max = count;
            }
            if count == 2 {
                two_kind_count += 1;
            }
        }
        return match (max, two_kind_count) {
            (4, _) => Rank::FourKind,
            (3, 1) => Rank::FullHouse,
            (3, _) => Rank::ThreeKind,
            (_, 2) => Rank::TwoPairs,
            (2, _) => Rank::OnePair,
            (_, _) => Rank::HighCard,
        };
    }
}

pub fn best_hand_in(cards: impl IntoIterator<Item = Card>) -> Hand {
    cards
        .into_iter()
        .combinations(5)
        .map(Hand::from)
        .max_by(|a, b| Rank::from(a).cmp(&Rank::from(b)))
        .unwrap()
}
