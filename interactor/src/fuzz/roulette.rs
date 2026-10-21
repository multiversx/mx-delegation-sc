//! Weighted random choice, the port of the Go fuzzer's `weightedroulette` package, plus
//! the one piece it did not need: drawing big integers.

use multiversx_sc_snippets::imports::*;
use rand::{Rng, RngExt, rngs::StdRng};

/// Picks one outcome, with probability proportional to its weight.
///
/// Outcomes with zero weight are never drawn, which is how a campaign turns an event off.
pub fn choose<T: Copy>(rng: &mut StdRng, outcomes: &[(u32, T)]) -> T {
    let total: u32 = outcomes.iter().map(|(weight, _)| weight).sum();
    assert!(
        total > 0,
        "every event weight is zero, nothing can be drawn"
    );

    let mut roll = rng.random_range(0..total);
    for (weight, outcome) in outcomes {
        if roll < *weight {
            return *outcome;
        }
        roll -= weight;
    }

    unreachable!("the roll is below the total weight, so some outcome must match")
}

/// A uniform big integer in `[0, max)`, the equivalent of Go's `big.Int.Rand`.
///
/// Drawing eight bytes more than `max` needs and reducing leaves a modulo bias below
/// 2^-64, which is far beyond anything a fuzz run could notice.
pub fn below(rng: &mut StdRng, max: &RustBigUint) -> RustBigUint {
    if max == &RustBigUint::default() {
        return RustBigUint::default();
    }

    let mut bytes = vec![0u8; max.to_bytes_be().len() + 8];
    rng.fill_bytes(&mut bytes);

    RustBigUint::from_bytes_be(&bytes) % max
}
