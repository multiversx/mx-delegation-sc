elrond_wasm::imports!();

#[elrond_wasm::derive::proxy]
pub trait GovernanceLegacyProxy {
    #[endpoint(delegateVote)]
    fn delegate_vote(
        &self,
        proposal_to_vote: u64,
        vote: &ManagedBuffer,
        voter: &ManagedAddress,
        user_stake: &BigUint,
    );
}
