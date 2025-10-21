#![feature(iter_array_chunks)]
#[macro_use]
mod cards;
mod eval;
mod rank;
mod state;

use cards::*;
//use rand::prelude::*;
use rank::*;
use state::*;

fn main() {
    let state = DeckState {
        hand: cards!("5h", "2h"),
        board: Board::Flop(cards!("3h", "4c", "jh")),
    };
    println!("HAND STRENGTH: {}", state.board_strength());
    println!("6 CARD STRENGTH: {}", state.strength_after_n_deals(1));
    println!("7 CARD STRENGTH: {}", state.strength_after_n_deals(2));
}
