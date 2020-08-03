
// use user_fund_storage::fund_type::*;

use crate::rewards::*;
use crate::settings::*;
use crate::user_unstake::*;
use user_fund_storage::user_data::*;
use user_fund_storage::fund_transf_module::*;
use user_fund_storage::fund_view_module::*;

imports!();

/// Contains logic for the owner to extract any unexpected balance that resides in the contract.
#[elrond_wasm_derive::module(UnexpectedBalanceModuleImpl)]
pub trait UnexpectedBalanceModule {

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(FundViewModuleImpl)]
    fn fund_view_module(&self) -> FundViewModuleImpl<T, BigInt, BigUint>;

    #[module(StakeSaleModuleImpl)]
    fn user_unstake(&self) -> StakeSaleModuleImpl<T, BigInt, BigUint>;

    /// Expected balance includes:
    /// - stake
    /// - unclaimed rewards (bulk uncomputed rewards + computed but unclaimed rewards; everything that was not yet sent to the delegators).
    /// Everything else is unexpected and can be withdrawn by the owner.
    /// This can come from someone accidentally sending ERD to the contract via direct transfer.
    #[view(getUnexpectedBalance)]
    fn get_unexpected_balance(&self) -> BigUint {
        let mut expected_balance = self.fund_view_module().get_total_stake();
        expected_balance += self.rewards().get_total_cumulated_rewards();
        expected_balance -= self.rewards().get_sent_rewards();

        self.get_sc_balance() - expected_balance
    }

    /// Used by owner to extract unexpected balance from contract.
    #[endpoint(withdrawUnexpectedBalance)]
    fn withdraw_unexpected_balance(&self) -> SCResult<()> {
        let caller = self.get_caller();
        if !self.settings().owner_called() {
            return sc_error!("only owner can withdraw unexpected balance");
        }

        let unexpected_balance = self.get_unexpected_balance();
        if unexpected_balance > 0 {
            self.send_tx(&caller, &unexpected_balance, "unexpected balance");
        }

        Ok(())
    }
}
