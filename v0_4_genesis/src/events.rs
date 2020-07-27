imports!();

/// Contains all events that can be emitted by the contract.
#[elrond_wasm_derive::module(EventsModuleImpl)]
pub trait EventsModule {

    #[event("0x0000000000000000000000000000000000000000000000000000000000000001")]
    fn stake_event(&self, delegator: &Address, amount: &BigUint);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000002")]
    fn unstake_event(&self, delegator: &Address, amount: &BigUint);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000003")]
    fn activation_ok_event(&self, _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000004")]
    fn activation_fail_event(&self, _reason: &[u8]);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000005")]
    fn deactivation_ok_event(&self, _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000006")]
    fn deactivation_fail_event(&self, _reason: &[u8]);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000007")]
    fn unbond_ok_event(&self, _data: ());

    #[event("0x0000000000000000000000000000000000000000000000000000000000000008")]
    fn unbond_fail_event(&self, _reason: &[u8]);

    #[event("0x0000000000000000000000000000000000000000000000000000000000000009")]
    fn purchase_stake_event(&self, seller: &Address, buyer: &Address, amount: &BigUint);

    #[event("0x000000000000000000000000000000000000000000000000000000000000000a")]
    fn claim_rewards_event(&self, user: &Address, amount: &BigUint);

}