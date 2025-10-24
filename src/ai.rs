use crate::{eval::Recommendation, state::Position};

pub fn decide(
    position: Position,
    hand_strength: f64,
    pot_odds: f64,
    num_opponents: usize,
    call_price: usize,
    pot: usize,
    stack: usize,
    blind: usize,
) -> (Recommendation, usize) {
    let bet_size = if call_price == 0 {
        (blind * 2).min(stack)
    } else {
        (call_price * 2).min(stack)
    };

    let raise_amount = bet_size - call_price;

    let fold_ev = 0.0;
    let call_ev = (pot as f64 * hand_strength) - (call_price as f64 * (1.0 - hand_strength));
    // Average pot size after raise
    let raise_pot = (0..num_opponents).fold(0, |total, calls| {
        total + pot + (calls * raise_amount) + bet_size
    }) as f64
        / (num_opponents as f64);
    let raise_ev = (raise_pot * hand_strength) - (bet_size as f64) * (1.0 - hand_strength);
    if raise_ev > call_ev && raise_ev > fold_ev {
        if bet_size >= stack {
            (Recommendation::AllIn, stack)
        } else {
            (Recommendation::Raise, bet_size)
        }
    } else if call_ev > fold_ev || hand_strength > pot_odds || call_price == 0 {
        (Recommendation::Call, call_price)
    } else {
        (Recommendation::Fold, 0)
    }
}
