use crate::cards::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandSuite {
    Suited,
    Unsuited,
}

pub struct OpeningHand([Face; 2], HandSuite);

impl From<[Card; 2]> for OpeningHand {
    fn from(value: [Card; 2]) -> Self {
        OpeningHand(
            [value[0].0, value[1].0],
            if value[0].1 == value[1].1 {
                HandSuite::Suited
            } else {
                HandSuite::Unsuited
            },
        )
    }
}

pub struct Preflop([[bool; 13]; 13]);

impl std::ops::Index<OpeningHand> for Preflop {
    type Output = bool;

    fn index(&self, index: OpeningHand) -> &Self::Output {
        let normalize = |f: Face| (if f == Face::Ace { 14 } else { f as usize } - 2);
        let mut arr = index.0.map(normalize);
        arr.sort();
        let [f1, f2] = arr;
        if index.1 == HandSuite::Suited {
            &self.0[f1][f2]
        } else {
            &self.0[f2][f1]
        }
    }
}

impl std::ops::IndexMut<OpeningHand> for Preflop {
    fn index_mut(&mut self, index: OpeningHand) -> &mut Self::Output {
        let normalize = |f: Face| (if f == Face::Ace { 14 } else { f as usize } - 2);
        let mut arr = index.0.map(normalize);
        arr.sort();
        let [f1, f2] = arr;
        if index.1 == HandSuite::Suited {
            &mut self.0[f1][f2]
        } else {
            &mut self.0[f2][f1]
        }
    }
}
