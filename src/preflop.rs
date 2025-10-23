use crate::{cards::*, eval::Recommendation, parse::Parse, state::Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandSuite {
    Suited,
    OffSuite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpeningHand([Face; 2], HandSuite);

impl std::fmt::Display for OpeningHand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0[0] == self.0[1] {
            write!(f, "{}{}", self.0[0], self.0[1])
        } else {
            write!(
                f,
                "{}{}{}",
                self.0[0],
                self.0[1],
                if self.1 == HandSuite::Suited {
                    's'
                } else {
                    'o'
                }
            )
        }
    }
}

impl From<[Card; 2]> for OpeningHand {
    fn from(value: [Card; 2]) -> Self {
        OpeningHand(
            [value[0].0, value[1].0],
            if value[0].1 == value[1].1 {
                HandSuite::Suited
            } else {
                HandSuite::OffSuite
            },
        )
    }
}

impl Parse for OpeningHand {
    fn parse(iter: &mut impl Iterator<Item = char>) -> Option<Self> {
        let faces = Face::parse_n(iter)?;
        let suited = if faces[0] == faces[1] {
            HandSuite::OffSuite
        } else {
            let next = iter.next()?;
            if next == 'o' {
                HandSuite::OffSuite
            } else if next == 's' {
                HandSuite::Suited
            } else {
                return None;
            }
        };
        Some(OpeningHand(faces, suited))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HandChart([[Recommendation; 13]; 13]);

impl HandChart {
    pub fn filled_with(rec: Recommendation) -> Self {
        let mut this = Self::default();
        for x in 0..this.0.len() {
            for y in 0..this.0.len() {
                this[(x, y)] = rec;
            }
        }
        this
    }

    /// Expected opening range of opponent hands
    pub fn opponent_expectation() -> Self {
        Self::new(
            Recommendation::Call,
            "22+,A2s+,K2s+,Q2s+,J3s+,T3s+,95s+,85s+,74s+,64s+,53s+,43s,A2o+,K5o+,Q8o+,J8o+,T7o+,97o+,87o",
        )
    }

    pub fn filter_hand(&self, to_filter: impl Into<OpeningHand>) -> bool {
        self[to_filter.into()] > Recommendation::Fold
    }

    pub fn new(rec: Recommendation, list: &str) -> Self {
        let mut this = Self::default();
        if list == "" {
            return this;
        }
        let iter = &mut list.chars();
        loop {
            let hand = OpeningHand::parse(iter).unwrap();
            match iter.next() {
                Some(',') => {
                    let (x, y) = index(hand);
                    this[(x, y)] = rec;
                }
                Some('+') if hand.0[0] == hand.0[1] => {
                    let (x, y) = index(hand);
                    for z in 0..=x {
                        this[(x - z, y - z)] = rec;
                    }
                    let next = iter.next();
                    if next.is_none() {
                        break;
                    }
                    if !next.is_some_and(|c| c == ',') {
                        panic!();
                    }
                }
                Some('+') => {
                    let (x, y) = index(hand);
                    if hand.1 == HandSuite::Suited {
                        for y in 0..=y {
                            this[(x, y)] = rec;
                        }
                    } else {
                        for x in 0..=x {
                            this[(x, y)] = rec;
                        }
                    }
                    let next = iter.next();
                    if next.is_none() {
                        break;
                    }
                    if !next.is_some_and(|c| c == ',') {
                        panic!();
                    }
                }
                Some(c) => panic!("Invalid character {c} in hand chart"),
                None => break,
            }
        }
        this
    }
}

fn index(idx: OpeningHand) -> (usize, usize) {
    let normalize = |f: Face| 12 - (if f == Face::Ace { 14 } else { f as usize } - 2);
    let mut arr = idx.0.map(normalize);
    arr.sort();
    let [f1, f2] = arr;
    if idx.1 == HandSuite::Suited {
        (f1, f2)
    } else {
        (f2, f1)
    }
}

impl<T> std::ops::Index<T> for HandChart
where
    T: Into<OpeningHand>,
{
    type Output = Recommendation;

    fn index(&self, idx: T) -> &Self::Output {
        let (a, b) = index(idx.into());
        &self.0[a][b]
    }
}

impl<T> std::ops::IndexMut<T> for HandChart
where
    T: Into<OpeningHand>,
{
    fn index_mut(&mut self, idx: T) -> &mut Self::Output {
        let (a, b) = index(idx.into());
        &mut self.0[a][b]
    }
}

impl std::ops::Index<(usize, usize)> for HandChart {
    type Output = Recommendation;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.0[index.0][index.1]
    }
}

impl std::ops::IndexMut<(usize, usize)> for HandChart {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.0[index.0][index.1]
    }
}

impl std::ops::BitOr for HandChart {
    type Output = HandChart;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        for x in 0..self.0.len() {
            for y in 0..self.0.len() {
                self.0[x][y] = self.0[x][y].max(rhs.0[x][y]);
            }
        }
        self
    }
}

impl std::fmt::Display for HandChart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            String::from_utf8_lossy(&self.0.iter().map(|b| b.map(Recommendation::symbol)).fold(
                vec![],
                |mut a, b| {
                    a.extend_from_slice(&b);
                    a.push('\n' as u8);
                    a
                }
            ),)
        )
    }
}

impl Position {
    /// Game theory optimal preflop chart
    pub fn gto_preflop(self) -> HandChart {
        use Recommendation::*;
        match self {
            Position::BigBlind => HandChart::filled_with(Call),
            Position::SmallBlind => HandChart::new(
                Call,
                "22+,A2s+,K2s+,Q2s+,J2s+,T2s+,92s+,82s+,72s+,62s+,52s+,42s+,32s,A2o+,K2o+,Q2o+,J2o+,T3o+,95o+,85o+,75o+,64o+,54o",
            ),
            Position::Button => HandChart::new(
                Raise,
                "22+,A2s+,K2s+,Q2s+,J3s+,T3s+,95s+,85s+,74s+,64s+,53s+,43s,A2o+,K5o+,Q8o+,J8o+,T7o+,97o+,87o",
            ),
            Position::Cutoff => HandChart::new(
                Raise,
                "22+,A2s+,K2s+,Q5s+,J7s+,T6s+,96s+,86s+,75s+,65s,54s,A5o+,K9o+,Q9o+,J9o+,T9o",
            ),
            Position::Hijack => HandChart::new(
                Raise,
                "22+,A2s+,K4s+,Q8s+,J8s+,T7s+,97s+,87s,76s,65s,54s,A8o+,KTo+,QTo+,JTo",
            ),
            Position::Lojack => HandChart::new(
                Raise,
                "33+,A2s+,K6s+,Q9s+,J8s+,T8s+,98s,87s,76s,A9o+,KTo+,QTo+",
            ),
            Position::UTG2 => {
                HandChart::new(Raise, "44+,A2s+,K8s+,Q9s+,J9s+,T8s+,98s,76s,ATo+,KTo+")
            }
            Position::UTG1 => HandChart::new(Raise, "66+,A3s+,K8s+,Q9s+,J9s+,T9s,98s,ATo+"),
            Position::UTG => HandChart::new(Raise, "66+,A3s+,K9s+,Q9s+,AJo+,KQo"),
        }
    }

    pub fn tall_preflop(self) -> HandChart {
        use Recommendation::*;
        match self {
            Position::BigBlind => HandChart::filled_with(Call),
            Position::SmallBlind => HandChart::new(Raise, ""),
            Position::Button => HandChart::new(Raise, ""),
            Position::Cutoff => HandChart::new(Raise, ""),
            Position::Hijack => HandChart::new(Raise, ""),
            Position::Lojack => HandChart::new(Raise, ""),
            Position::UTG2 => HandChart::new(Raise, ""),
            Position::UTG1 => HandChart::new(Raise, ""),
            Position::UTG => HandChart::new(Raise, ""),
        }
    }

    /// Preflop chart for short stacks
    pub fn short_preflop(self) -> HandChart {
        use Recommendation::*;
        match self {
            Position::BigBlind => HandChart::filled_with(Call),
            Position::SmallBlind => HandChart::new(
                AllIn,
                "22+,A2s+,K2s+,Q2s+,J2s+,T2s+,92s+,82s+,72s+,62s+,52s+,42s+,32s,A2o+,K2o+,Q2o+,J2o+,T4o+,95o+,86o+,75o+,65o",
            ),
            Position::Button => HandChart::new(
                AllIn,
                "22+,A2s+,K2s+,Q5s+,J6s+,T6s+,97s+,87s,A2o+,K7o+,QTo+,JTo",
            ),
            Position::Cutoff => {
                HandChart::new(AllIn, "22+,A2s+,K6s+,Q8s+,J8s+,T8s+,98s,A2o+,KTo+,QTo+,JTo")
            }
            Position::Hijack => {
                HandChart::new(AllIn, "22+,A2s+,K9s+,Q9s+,J9s+,T8s+,98s,A3o+,KTo+,QJo,JTo")
            }
            Position::Lojack => {
                HandChart::new(AllIn, "22+,A2s+,K9s+,Q9s+,J9s+,T9s,98s,A7o+,KTo+,QJo")
            }
            Position::UTG2 => HandChart::new(AllIn, "33+,A2s+,K9s+,Q9s+,J9s+,T9s,A9o+,KJo+,QJo"),
            Position::UTG1 => HandChart::new(AllIn, "33+,A2s+,K9s+,Q9s+,J9s+,T9s,A9o+,KJo+,QJo"),
            Position::UTG => HandChart::new(AllIn, "44+,A4s+,K9s+,QTs+,J9s+,T9s,A9o+,KJo+"),
        }
    }
}
