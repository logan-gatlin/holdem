use itertools::Itertools;

macro_rules! card {
    ($e:expr) => {
        $e.parse::<crate::cards::Card>().unwrap()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suite {
    Heart = 1,
    Spade = 2,
    Diamond = 3,
    Club = 4,
}

impl std::str::FromStr for Suite {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "h" | "H" => Ok(Self::Heart),
            "s" | "S" => Ok(Self::Spade),
            "d" | "D" => Ok(Self::Diamond),
            "c" | "C" => Ok(Self::Club),
            _ => Err(()),
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

impl std::str::FromStr for Face {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "a" | "A" | "1" => Ok(Self::Ace),
            "2" => Ok(Self::Two),
            "3" => Ok(Self::Three),
            "4" => Ok(Self::Four),
            "5" => Ok(Self::Five),
            "6" => Ok(Self::Six),
            "7" => Ok(Self::Seven),
            "8" => Ok(Self::Eight),
            "9" => Ok(Self::Nine),
            "t" | "T" => Ok(Self::Ten),
            "j" | "J" => Ok(Self::Jack),
            "q" | "Q" => Ok(Self::Queen),
            "k" | "K" => Ok(Self::King),
            _ => Err(()),
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

impl std::str::FromStr for Card {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.chars().filter(|c| !c.is_whitespace());
        let face = String::from(iter.next().ok_or(())?).parse()?;
        let suite = String::from(iter.next().ok_or(())?).parse()?;
        Ok(Card(face, suite))
    }
}

pub fn parse_cards(input: &str) -> Option<Vec<Card>> {
    input
        .chars()
        .filter(|c| !c.is_whitespace())
        .array_chunks()
        .map(|[a, b]| format!("{a}{b}").parse::<Card>())
        .try_collect()
        .ok()
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
            crate::rank::Rank::from(self)
        )
    }
}

impl std::str::FromStr for Hand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cards: Vec<_> = s
            .chars()
            .filter(|c| !c.is_whitespace())
            .array_chunks()
            .map(|[a, b]| format!("{a}{b}").parse::<Card>())
            .try_collect()?;
        Ok(Hand::from(cards))
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
