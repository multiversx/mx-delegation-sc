
use crate::user_stake_state::*;

use crate::rewards::*;
use crate::settings::*;
use crate::user_data::*;

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



    /// Expected balance includes:
    /// - stake
    /// - unclaimed rewards (bulk uncomputed rewards + computed but unclaimed rewards; everything that was not yet sent to the delegators).
    /// Everything else is unexpected and can be withdrawn by the owner.
    /// This can come from someone accidentally sending ERD to the contract via direct transfer.
    #[view]
    fn getUnexpectedBalance(&self) -> BigUint {
        let mut expected_balance = self.user_data().get_user_stake_of_type(USER_STAKE_TOTALS_ID, UserStakeState::Inactive);
        expected_balance += self.user_data().get_user_stake_of_type(USER_STAKE_TOTALS_ID, UserStakeState::WithdrawOnly);
        expected_balance += self.rewards().getTotalCumulatedRewards();
        expected_balance -= self.rewards().get_sent_rewards();

        self.get_sc_balance() - expected_balance
    }

    /// Used by owner to extract unexpected balance from contract.
    #[endpoint(withdrawUnexpectedBalance)]
    fn withdraw_unexpected_balance(&self) -> Result<(), SCError> {
        let caller = self.get_caller();
        if &caller != &self.settings().getContractOwner() {
            return sc_error!("only owner can withdraw unexpected balance");
        }

        let unexpected_balance = self.getUnexpectedBalance();
        if unexpected_balance > 0 {
            self.send_tx(&caller, &unexpected_balance, "unexpected balance");
        }

        Ok(())
    }
}
