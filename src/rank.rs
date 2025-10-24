#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RankKind {
    HighCard,
    OnePair,
    TwoPairs,
    ThreeKind,
    Straight,
    Flush,
    FullHouse,
    FourKind,
    StraightFlush,
    RoyalFlush,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ranking {
    kind: RankKind,
    hand_descending: [usize; 5],
}

impl PartialOrd for Ranking {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::*;
        match self.kind.cmp(&other.kind) {
            Greater => Some(Greater),
            Less => Some(Less),
            Equal => Some(self.hand_descending.cmp(&other.hand_descending)),
        }
    }
}

impl Ord for Ranking {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl std::fmt::Display for RankKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RankKind::HighCard => "High card",
                RankKind::OnePair => "Pair",
                RankKind::TwoPairs => "Two pairs",
                RankKind::ThreeKind => "Three of a kind",
                RankKind::Straight => "Straight",
                RankKind::Flush => "Flush",
                RankKind::FullHouse => "Full house",
                RankKind::FourKind => "Four of a kind",
                RankKind::StraightFlush => "Straight flush",
                RankKind::RoyalFlush => "Royal flush",
            }
        )
    }
}

impl std::fmt::Display for Ranking {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

use itertools::Itertools;

use crate::cards::*;

impl From<Hand> for Ranking {
    fn from(value: Hand) -> Self {
        Self::from(&value)
    }
}

impl From<&Hand> for Ranking {
    fn from(hand: &Hand) -> Self {
        use Face::*;
        let mut face_values = hand.face_values();
        face_values.sort();
        let mut hand_descending = face_values.clone();
        hand_descending.reverse();
        let r = |kind| Ranking {
            kind,
            hand_descending,
        };
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
            (true, true) if is_high_straight => {
                return r(RankKind::RoyalFlush);
            }
            (true, true) => return r(RankKind::StraightFlush),
            (true, false) => return r(RankKind::Straight),
            (false, true) => return r(RankKind::Flush),
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
        return r(match (max, two_kind_count) {
            (4, _) => RankKind::FourKind,
            (3, 1) => RankKind::FullHouse,
            (3, _) => RankKind::ThreeKind,
            (_, 2) => RankKind::TwoPairs,
            (2, _) => RankKind::OnePair,
            (_, _) => RankKind::HighCard,
        });
    }
}

pub fn best_hand_in(cards: impl IntoIterator<Item = Card>) -> Hand {
    cards
        .into_iter()
        .combinations(5)
        .map(Hand::from)
        .max_by(|a, b| Ranking::from(a).cmp(&Ranking::from(b)))
        .unwrap()
}
