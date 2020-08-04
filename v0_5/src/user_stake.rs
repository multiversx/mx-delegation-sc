
use crate::events::*;
use crate::pause::*;
use crate::settings::*;
use user_fund_storage::user_data::*;
use user_fund_storage::fund_transf_module::*;
use user_fund_storage::fund_view_module::*;
use user_fund_storage::types::*;

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

    #[module(PauseModuleImpl)]
    fn pause(&self) -> PauseModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

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

        // log staking event
        self.events().stake_event(&caller, &payment);

        // create stake funds
        let amount_to_stake = payment.clone();
        self.fund_transf_module().create_waiting(user_id, payment);

        // get total unstaked
        let total_unstaked = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked);
        if total_unstaked == 0 {
            return Ok(());
        }
        let swappable = core::cmp::min(&amount_to_stake, &total_unstaked);

        // swap unStaked to deferred payment
        sc_try!(self.fund_transf_module().swap_unstaked_to_deferred_payment(&swappable));

        // swap waiting to active
        sc_try!(self.fund_transf_module().swap_active_with_waiting_transf(&swappable));

        Ok(())
    }

    /// Delegate stake to the smart contract. 
    /// Stake is initially inactive, so does it not produce rewards.
    #[payable]
    #[endpoint(stake)]
    fn stake_endpoint(&self, #[payment] payment: BigUint) -> SCResult<()> {
        if self.pause().is_staking_paused() {
            return sc_error!("staking paused");
        }
        if payment < self.settings().get_minimum_stake() {
            return sc_error!("cannot stake less than minimum stake")
        }

        self.process_stake(payment)
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
        sc_try!(self.fund_transf_module().liquidate_free_stake(user_id, &mut amount_to_unstake));
        if amount_to_unstake > 0 {
            return sc_error!("cannot withdraw more than inactive stake");
        }

        sc_try!(self.validate_total_user_stake(user_id));

        // send stake to delegator
        self.send_tx(&caller, &amount, "delegation withdraw inactive stake");

        // log
        self.events().unstake_event(&caller, &amount);

        Ok(())
    }

    fn validate_total_user_stake(&self, user_id: usize) -> SCResult<()> {
        let user_total = self.fund_view_module().get_user_total_stake(user_id);
        if user_total > 0 && user_total < self.settings().get_minimum_stake() {
            return sc_error!("cannot have less stake than minimum stake");
        }
        Ok(())
    }

    fn validate_owner_stake_share(&self) -> SCResult<()> {
        // owner total stake / contract total stake < owner_min_stake_share / 10000
        // reordered to avoid divisions
        if self.fund_view_module().get_user_total_stake(OWNER_USER_ID) * BigUint::from(PERCENTAGE_DENOMINATOR) <
            self.fund_view_module().get_total_stake() * self.settings().get_owner_min_stake_share() {
                return sc_error!("owner doesn't have enough stake in the contract");
            }
        Ok(())
    }
     
}
