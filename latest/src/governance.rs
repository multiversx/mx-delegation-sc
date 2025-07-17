use multiversx_sc::imports::*;
use user_fund_storage::types::FundType;

use super::governance_sc_proxy::GovernanceSCLocalProxy;

/// Contains logic to forward governance votes to the governance system SC.
#[multiversx_sc::derive::module]
pub trait GovernanceModule:
    user_fund_storage::user_data::UserDataModule
    + user_fund_storage::fund_module::FundModule
    + user_fund_storage::fund_view_module::FundViewModule
{
    #[endpoint(delegateVote)]
    fn delegate_vote(&self, proposal_to_vote: u64, vote: ManagedBuffer) {
        let voter = self.blockchain().get_caller();

        let staked_balance = self.get_user_stake_of_type_by_address(&voter, FundType::Active);

        self.tx()
            .to(GovernanceSystemSCAddress)
            .typed(GovernanceSCLocalProxy)
            .delegate_vote(&proposal_to_vote, &vote, &voter, &staked_balance)
            .callback(
                self.callbacks()
                    .delegate_vote_callback(&voter, proposal_to_vote, &vote),
            )
            .async_call_and_exit();
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
        self.get_user_stake_of_type_by_address(&voter, FundType::Active)
    }
}
