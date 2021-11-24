elrond_wasm::imports!();

/// Contains all events that can be emitted by the contract.
#[elrond_wasm::derive::module]
pub trait EventsModule {
    // #[legacy_event("0x0000000000000000000000000000000000000000000000000000000000000001")]
    #[event("userStake")]
    fn stake_event(&self, #[indexed] delegator: &ManagedAddress, amount: &BigUint);

    // #[legacy_event("0x0000000000000000000000000000000000000000000000000000000000000002")]
    #[event("userUnstake")]
    fn unstake_event(&self, #[indexed] delegator: &ManagedAddress, amount: &BigUint);

    // #[legacy_event("0x0000000000000000000000000000000000000000000000000000000000000003")]
    #[event("nodeStakeOk")]
    fn stake_node_ok_event(&self);

    // #[legacy_event("0x0000000000000000000000000000000000000000000000000000000000000004")]
    #[event("nodeStakeFail")]
    fn stake_node_fail_event(&self, reason: &[u8]);

    // #[legacy_event("0x0000000000000000000000000000000000000000000000000000000000000005")]
    #[event("nodeUnstakeOk")]
    fn unstake_node_ok_event(&self);

    // #[legacy_event("0x0000000000000000000000000000000000000000000000000000000000000006")]
    #[event("nodeUnstakeFail")]
    fn unstake_node_fail_event(&self, reason: &[u8]);

    // #[legacy_event("0x0000000000000000000000000000000000000000000000000000000000000007")]
    #[event("nodeUnbondOk")]
    fn unbond_node_ok_event(&self);

    // #[legacy_event("0x0000000000000000000000000000000000000000000000000000000000000008")]
    #[event("nodeUnbondFail")]
    fn unbond_node_fail_event(&self, reason: &[u8]);

    // #[legacy_event("0x0000000000000000000000000000000000000000000000000000000000000009")]
    #[event("userClaimRewards")]
    fn claim_rewards_event(&self, #[indexed] user: &ManagedAddress, amount: &BigUint);
}
