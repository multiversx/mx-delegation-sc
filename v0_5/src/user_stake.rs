
use crate::events::*;
use crate::pause::*;
use crate::rewards::*;
use crate::settings::*;
use crate::reset_checkpoints::*;
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

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    #[module(ResetCheckpointsModuleImpl)]
    fn reset_checkpoints(&self) -> ResetCheckpointsModuleImpl<T, BigInt, BigUint>;

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

        let total_staked = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
        let total_delegation_cap = self.settings().get_total_delegation_cap();

        let total_free = total_delegation_cap - total_staked;
        if total_free == 0 {
            return Ok(());
        }
        let swappable = core::cmp::min(&amount_to_stake, &total_free);

        // swap unStaked to deferred payment
        let total_unstaked = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked);
        if total_unstaked > 0 {
            let all_unstaked = &total_unstaked;
            let unstaked_swappable = core::cmp::min(&swappable, &all_unstaked);
            let _ = self.fund_transf_module().swap_unstaked_to_deferred_payment(&unstaked_swappable, || false);
        }

        // swap waiting to active
        let (affected_users, remaining) = self.fund_transf_module().swap_waiting_to_active(&swappable, || false);
        if remaining > 0 {
            return sc_error!("error swapping waiting to active")
        }

        for user_id in affected_users.iter() {
            self.rewards().compute_one_user_reward(*user_id);
        }

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
        if self.reset_checkpoints().get_global_check_point_in_progress() {
            return sc_error!("staking is temporarily paused as checkpoint is reset")
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

        let mut amount_to_withdraw = amount.clone();
        sc_try!(self.fund_transf_module().liquidate_free_stake(user_id, &mut amount_to_withdraw));
        if amount_to_withdraw > 0 {
            return sc_error!("cannot withdraw more than inactive stake");
        }

        sc_try!(self.validate_total_user_stake(user_id));

        // send stake to delegator
        self.send_tx(&caller, &amount, "delegation withdraw inactive stake");

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
        if self.fund_view_module().get_user_stake_of_type(OWNER_USER_ID, FundType::Active) * BigUint::from(PERCENTAGE_DENOMINATOR) <
        self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active) * self.settings().get_owner_min_stake_share() {
                return sc_error!("owner doesn't have enough stake in the contract");
            }
        Ok(())
    }
    
}
