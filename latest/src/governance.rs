use hex_literal::hex;
use user_fund_storage::types::FundType;

use crate::governance_legacy_proxy;

elrond_wasm::imports!();

const GOVERNANCE_SC_ADDRESS_BYTES: [u8; 32] =
    hex!("000000000000000000010000000000000000000000000000000000000003ffff");

/// Contains logic to forward governance votes to the governance system SC.
#[elrond_wasm::derive::module]
pub trait GovernanceModule:
    user_fund_storage::user_data::UserDataModule
    + user_fund_storage::fund_module::FundModule
    + user_fund_storage::fund_view_module::FundViewModule
{
    #[proxy]
    fn governance_proxy(&self, to: ManagedAddress) -> governance_legacy_proxy::Proxy<Self::Api>;

    #[endpoint(delegateVote)]
    fn delegate_vote(&self, proposal_to_vote: u64, vote: ManagedBuffer) {
        let voter = self.blockchain().get_caller();

        let staked_balance =
            self.get_user_stake_of_type_by_address(voter.clone(), FundType::Active);

        self.governance_proxy(ManagedAddress::new_from_bytes(&GOVERNANCE_SC_ADDRESS_BYTES))
            .delegate_vote(proposal_to_vote, &vote, &voter, &staked_balance)
            .async_call()
            .with_callback(
                self.callbacks()
                    .delegate_vote_callback(&voter, proposal_to_vote, &vote),
            )
            .call_and_exit()
    }

    #[callback]
    fn delegate_vote_callback(
        &self,
        voter: &ManagedAddress,
        proposal_to_vote: u64,
        vote: &ManagedBuffer,
        #[call_result] call_result: ManagedAsyncCallResult<()>,
    ) {
        match call_result {
            ManagedAsyncCallResult::Ok(()) => {
                self.delegate_vote_success_event(voter, proposal_to_vote, vote);
            }
            ManagedAsyncCallResult::Err(error) => {
                self.delegate_vote_error_event(voter, proposal_to_vote, vote, &error.err_msg);
            }
        }
    }

    #[event("delegateVoteSuccess")]
    fn delegate_vote_success_event(
        &self,
        #[indexed] voter: &ManagedAddress,
        #[indexed] proposal_to_vote: u64,
        #[indexed] vote: &ManagedBuffer,
    );

    #[event("delegateVoteError")]
    fn delegate_vote_error_event(
        &self,
        #[indexed] voter: &ManagedAddress,
        #[indexed] proposal_to_vote: u64,
        #[indexed] vote: &ManagedBuffer,
        error_msg: &ManagedBuffer,
    );

    /// Voting power of a single user, based on their active stake.
    #[view(getVotingPower)]
    fn get_voting_power(&self, voter: ManagedAddress) -> BigUint {
        self.get_user_stake_of_type_by_address(voter, FundType::Active)
    }
}
