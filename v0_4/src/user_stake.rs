
use crate::events::*;
use crate::pause::*;
use crate::settings::*;
use crate::user_data::*;
use crate::fund_transf_module::*;
use crate::fund_view_module::*;
// use crate::settings::*;

imports!();

/// Contains endpoints for staking/withdrawing stake.
#[elrond_wasm_derive::module(UserStakeModuleImpl)]
pub trait UserStakeModule {

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(FundViewModuleImpl)]
    fn fund_view_module(&self) -> FundViewModuleImpl<T, BigInt, BigUint>;

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(PauseModuleImpl)]
    fn pause(&self) -> PauseModuleImpl<T, BigInt, BigUint>;

    /// Delegate stake to the smart contract. 
    /// Stake is initially inactive, so does it not produce rewards.
    #[payable]
    #[endpoint(stake)]
    fn stake_endpoint(&self, #[payment] payment: BigUint) -> SCResult<()> {
        if self.pause().is_staking_paused() {
            return sc_error!("staking paused");
        }
        
        if payment == 0 {
            return Ok(());
        }

        self.process_stake(payment)
    }

    fn process_stake(&self, payment: BigUint) -> SCResult<()> {
        // get user id or create user
        // we use user id as an intermediate identifier between user address and data,
        // because we might at some point need to iterate over all user data
        let caller = self.get_caller();
        let mut user_id = self.user_data().get_user_id(&caller);
        if user_id == 0 {
            user_id = self.user_data().new_user();
            self.user_data().set_user_id(&caller, user_id);
        }

        // create stake funds
        self.fund_transf_module().create_free_stake(user_id, &payment);

        // log staking event
        self.events().stake_event(&caller, &payment);

        Ok(())
    }

    // WITHDRAW INACTIVE

    #[endpoint(withdrawInactiveStake)]
    fn withdraw_inactive_stake(&self, amount: BigUint) -> SCResult<()> {
        if amount == 0 {
            return Ok(());
        }

        let caller = self.get_caller();
        let user_id = self.user_data().get_user_id(&caller);
        if user_id == 0 {
            return sc_error!("only delegators can withdraw inactive stake");
        }

        let mut amount_to_unstake = amount.clone();
        self.fund_transf_module().liquidate_free_stake(user_id, &mut amount_to_unstake);
        if amount_to_unstake > 0 {
            return sc_error!("cannot withdraw more than inactive stake");
        }

        sc_try!(self.fund_view_module().validate_total_user_stake(user_id));

        // send stake to delegator
        self.send_tx(&caller, &amount, "delegation withdraw inactive stake");

        // log
        self.events().unstake_event(&caller, &amount);

        Ok(())
    }
    
}
