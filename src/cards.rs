use itertools::Itertools;

use crate::parse::Parse;

macro_rules! card {
    ($e:expr) => {
        <Card as crate::parse::Parse>::parse(&mut $e.chars()).unwrap()
    };
}

macro_rules! cards {
    ($($e:expr),*) => {
        [$(card!($e),)*]
    }
}

pub fn deck() -> Vec<Card> {
    Face::ALL
        .into_iter()
        .cartesian_product(Suite::ALL)
        .map(|(face, suite)| Card(face, suite))
        .collect::<Vec<_>>()
}

pub fn deck_without(cards: impl IntoIterator<Item = Card>) -> Vec<Card> {
    let cards = cards.into_iter().collect::<Vec<_>>();
    deck().into_iter().filter(|c| !cards.contains(c)).collect()
}

pub fn unique_open_hands() -> Vec<[Card; 2]> {
    let mut cards = vec![];
    for f1 in Face::ALL {
        for f2 in Face::ALL {
            cards.push(if f1 <= f2 {
                [Card(f1, Suite::Heart), Card(f2, Suite::Spade)]
            } else {
                [Card(f1, Suite::Heart), Card(f2, Suite::Heart)]
            })
        }
    }
    cards
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suite {
    Heart = 1,
    Spade = 2,
    Diamond = 3,
    Club = 4,
}

impl Parse for Suite {
    fn parse(iter: &mut impl Iterator<Item = char>) -> Option<Self> {
        match iter.next()? {
            'h' | 'H' => Some(Self::Heart),
            's' | 'S' => Some(Self::Spade),
            'd' | 'D' => Some(Self::Diamond),
            'c' | 'C' => Some(Self::Club),
            _ => return None,
        }
    }
}

impl std::fmt::Display for Suite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Suite::Heart => "H",
                Suite::Diamond => "D",
                Suite::Spade => "S",
                Suite::Club => "C",
            }
        )
    }
}

impl Suite {
    pub const ALL: [Suite; 4] = [Self::Heart, Self::Spade, Self::Diamond, Self::Club];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Face {
    Ace = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
}

impl std::fmt::Display for Face {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Face::Two => "2",
                Face::Three => "3",
                Face::Four => "4",
                Face::Five => "5",
                Face::Six => "6",
                Face::Seven => "7",
                Face::Eight => "8",
                Face::Nine => "9",
                Face::Ten => "T",
                Face::Jack => "J",
                Face::Queen => "Q",
                Face::King => "K",
                Face::Ace => "A",
            }
        )
    }
}

impl Parse for Face {
    fn parse(iter: &mut impl Iterator<Item = char>) -> Option<Self> {
        match iter.next()? {
            'a' | 'A' | '1' => Some(Self::Ace),
            '2' => Some(Self::Two),
            '3' => Some(Self::Three),
            '4' => Some(Self::Four),
            '5' => Some(Self::Five),
            '6' => Some(Self::Six),
            '7' => Some(Self::Seven),
            '8' => Some(Self::Eight),
            '9' => Some(Self::Nine),
            't' | 'T' => Some(Self::Ten),
            'j' | 'J' => Some(Self::Jack),
            'q' | 'Q' => Some(Self::Queen),
            'k' | 'K' => Some(Self::King),
            _ => return None,
        }
    }
}

impl Face {
    pub const ALL: [Face; 13] = [
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
        Self::Jack,
        Self::Queen,
        Self::King,
        Self::Ace,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card(pub Face, pub Suite);

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.1)
    }
}

impl Parse for Card {
    fn parse(iter: &mut impl Iterator<Item = char>) -> Option<Self> {
        let face = Face::parse(iter)?;
        let suite = Suite::parse(iter)?;
        Some(Card(face, suite))
    }
}

impl Parse for Vec<Card> {
    fn parse(iter: &mut impl Iterator<Item = char>) -> Option<Self> {
        let mut buf = vec![];
        while let Some(c) = Card::parse(iter) {
            buf.push(c);
        }
        if iter.next().is_some() {
            return None;
        }
        Some(buf)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Hand(pub [Card; 5]);

impl std::fmt::Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut faces = self.faces();
        faces.sort();
        write!(
            f,
            "{}{}{}{}{} [{}]",
            faces[0],
            faces[1],
            faces[2],
            faces[3],
            faces[4],
            crate::rank::Ranking::from(self)
        )
    }
}

impl Parse for Hand {
    fn parse(iter: &mut impl Iterator<Item = char>) -> Option<Self> {
        Some(Hand(Card::parse_n(iter)?))
    }
}

impl<T> From<T> for Hand
where
    T: IntoIterator<Item = Card>,
{
    fn from(value: T) -> Self {
        let cards = value.into_iter().collect::<Vec<_>>();
        assert!(cards.len() == 5);
        Hand([cards[0], cards[1], cards[2], cards[3], cards[4]])
    }
}

impl Hand {
    pub fn faces(&self) -> [Face; 5] {
        self.0.map(|c| c.0)
    }

    pub fn face_values(&self) -> [usize; 5] {
        self.0.map(|c| c.0 as usize)
    }

    pub fn faces_iter(&self) -> impl Iterator<Item = Face> {
        self.0.iter().map(|c| c.0)
    }

    pub fn suites(&self) -> [Suite; 5] {
        self.0.map(|c| c.1)
    }

    pub fn suites_iter(&self) -> impl Iterator<Item = Suite> {
        self.0.iter().map(|c| c.1)
    }
}
