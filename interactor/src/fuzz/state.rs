//! What a run accumulates, to compare against the contract when it ends.

use multiversx_sc_snippets::imports::*;

#[derive(Default)]
pub struct FuzzState {
    /// Everything ever sent into the contract as stake.
    pub total_stake_added: RustBigUint,

    /// Every reward injected, which is also what the contract's own cumulated counter
    /// should report.
    pub total_rewards: RustBigUint,

    /// Rewards claimed and forwarded to the rewards sink.
    pub total_claimed: RustBigUint,

    /// Stake unbonded and forwarded to the withdraw sink.
    pub total_withdrawn: RustBigUint,

    /// How many node registrations were requested.
    pub nodes_added: usize,
}
