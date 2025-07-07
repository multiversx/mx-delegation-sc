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
            .delegate_vote(proposal_to_vote, vote, voter, staked_balance)
            .async_call_and_exit();
    }
}
