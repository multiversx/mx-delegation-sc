//! The random event table: what a step can be, how one is drawn, and what it logs.
//!
//! The port of `generateRandomEvent` in `fuzzDelegation_test.go`.

use core::fmt;

use multiversx_sc_snippets::imports::*;
use rand::{RngExt, rngs::StdRng};
use serde_json::{Value, json};

use crate::{
    fuzz::{bounds::Bounds, roulette},
    test_env::EventWeights,
};

/// One step of a fuzz run, with its parameters already drawn.
///
/// Holding the parameters rather than redrawing them at execution time is what makes the
/// action log a faithful record: replaying the log reproduces the run even though the
/// chain simulator's block timing does not reproduce from a seed alone.
#[derive(Clone, Debug)]
pub enum Event {
    AdvanceBlocks(u64),
    AddNodes(usize),
    RemoveNodes(usize),
    Stake {
        delegator: usize,
        amount: RustBigUint,
    },
    AddRewards(RustBigUint),
    ClaimRewards {
        delegator: usize,
    },
    Unstake {
        delegator: usize,
        amount: RustBigUint,
    },
    Unbond {
        delegator: usize,
    },
    ModifyCap(RustBigUint),
    SetServiceFee(usize),
    DustCleanup(RustBigUint),
}

/// The event kinds, used to draw one before its parameters exist.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EventKind {
    AdvanceBlocks,
    AddNodes,
    RemoveNodes,
    Stake,
    AddRewards,
    ClaimRewards,
    Unstake,
    Unbond,
    ModifyCap,
    SetServiceFee,
    DustCleanup,
}

impl Event {
    pub fn random(rng: &mut StdRng, weights: &EventWeights, bounds: &Bounds) -> Self {
        let kind = roulette::choose(
            rng,
            &[
                (weights.advance_blocks, EventKind::AdvanceBlocks),
                (weights.add_nodes, EventKind::AddNodes),
                (weights.remove_nodes, EventKind::RemoveNodes),
                (weights.stake, EventKind::Stake),
                (weights.add_rewards, EventKind::AddRewards),
                (weights.claim_rewards, EventKind::ClaimRewards),
                (weights.unstake, EventKind::Unstake),
                (weights.unbond, EventKind::Unbond),
                (weights.modify_cap, EventKind::ModifyCap),
                (weights.set_service_fee, EventKind::SetServiceFee),
                (weights.dust_cleanup, EventKind::DustCleanup),
            ],
        );

        // Delegator 0 is the owner, and the Go fuzzer included it in the draw.
        let delegator = |rng: &mut StdRng| rng.random_range(0..=bounds.num_delegators);

        match kind {
            EventKind::AdvanceBlocks => {
                Event::AdvanceBlocks(rng.random_range(1..=bounds.block_advance_max.max(1)))
            }
            EventKind::AddNodes => Event::AddNodes(rng.random_range(0..=bounds.max_nodes_added)),
            EventKind::RemoveNodes => {
                Event::RemoveNodes(rng.random_range(0..=bounds.max_nodes_removed))
            }
            EventKind::Stake => Event::Stake {
                delegator: delegator(rng),
                amount: roulette::below(rng, &bounds.max_stake),
            },
            EventKind::AddRewards => Event::AddRewards(roulette::below(rng, &bounds.max_reward)),
            EventKind::ClaimRewards => Event::ClaimRewards {
                delegator: delegator(rng),
            },
            EventKind::Unstake => Event::Unstake {
                delegator: delegator(rng),
                amount: roulette::below(rng, &bounds.max_stake),
            },
            EventKind::Unbond => Event::Unbond {
                delegator: delegator(rng),
            },
            EventKind::ModifyCap => {
                Event::ModifyCap(roulette::below(rng, &bounds.max_delegation_cap))
            }
            EventKind::SetServiceFee => {
                Event::SetServiceFee(rng.random_range(0..bounds.max_service_fee))
            }
            EventKind::DustCleanup => Event::DustCleanup(roulette::below(rng, &bounds.max_dust)),
        }
    }

    /// Whether the Go fuzzer checked the contract's invariants after this kind of event.
    ///
    /// It deliberately skipped the ones that cannot change the balance between owner stake
    /// and total stake: removing nodes, claiming, and unbonding.
    pub fn checks_invariants(&self) -> bool {
        !matches!(
            self,
            Event::RemoveNodes(_) | Event::ClaimRewards { .. } | Event::Unbond { .. }
        )
    }

    /// The line written to the action log, which a later `--replay` reads back.
    pub fn log_value(&self) -> Value {
        match self {
            Event::AdvanceBlocks(blocks) => json!({ "event": "advance_blocks", "blocks": blocks }),
            Event::AddNodes(count) => json!({ "event": "add_nodes", "count": count }),
            Event::RemoveNodes(count) => json!({ "event": "remove_nodes", "count": count }),
            Event::Stake { delegator, amount } => json!({
                "event": "stake", "delegator": delegator, "amount": amount.to_string(),
            }),
            Event::AddRewards(amount) => json!({
                "event": "add_rewards", "amount": amount.to_string(),
            }),
            Event::ClaimRewards { delegator } => json!({
                "event": "claim_rewards", "delegator": delegator,
            }),
            Event::Unstake { delegator, amount } => json!({
                "event": "unstake", "delegator": delegator, "amount": amount.to_string(),
            }),
            Event::Unbond { delegator } => json!({ "event": "unbond", "delegator": delegator }),
            Event::ModifyCap(cap) => json!({ "event": "modify_cap", "cap": cap.to_string() }),
            Event::SetServiceFee(fee) => json!({ "event": "set_service_fee", "fee": fee }),
            Event::DustCleanup(limit) => json!({
                "event": "dust_cleanup", "limit": limit.to_string(),
            }),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::AdvanceBlocks(blocks) => write!(f, "advance {blocks} block(s)"),
            Event::AddNodes(count) => write!(f, "add {count} node(s)"),
            Event::RemoveNodes(count) => write!(f, "remove {count} node(s)"),
            Event::Stake { delegator, .. } => write!(f, "stake, delegator {delegator}"),
            Event::AddRewards(_) => write!(f, "add rewards"),
            Event::ClaimRewards { delegator } => write!(f, "claim rewards, delegator {delegator}"),
            Event::Unstake { delegator, .. } => write!(f, "unStake, delegator {delegator}"),
            Event::Unbond { delegator } => write!(f, "unBond, delegator {delegator}"),
            Event::ModifyCap(_) => write!(f, "modify delegation cap"),
            Event::SetServiceFee(fee) => write!(f, "set service fee to {fee}"),
            Event::DustCleanup(_) => write!(f, "dust cleanup"),
        }
    }
}
