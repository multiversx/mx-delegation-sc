
use crate::events::*;
use crate::rewards::*;
use crate::settings::*;
use crate::reset_checkpoints::*;
use user_fund_storage::user_data::*;
use user_fund_storage::fund_transf_module::*;
use user_fund_storage::fund_view_module::*;
use user_fund_storage::types::*;
use elrond_wasm_module_pause::*;

use core::num::NonZeroUsize;

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
        self.fund_transf_module().create_waiting(user_id, payment);

        // check invariant
        sc_try!(self.validate_delegation_cap_invariant());

        // move funds around
        self.use_waiting_to_replace_unstaked()
    }

    /// The contract can be either overstaked (waiting > 0) or understaked (unstaked > 0).
    /// Cannot have both, since waiting should "cancel out" the unstaked.
    /// This operation does this. It takes min(waiting, unstaked) and converts this amount
    /// from waiting to active and from unstaked to deferred payment. 
    /// Note that this operation preserves the invariant that active + unstaked == delegation_cap.
    fn use_waiting_to_replace_unstaked(&self) -> SCResult<()> {
        let total_waiting = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Waiting);
        let mut total_unstaked = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked);

        if self.settings().is_bootstrap_mode() {
            // bootstrap mode!
            // the total delegation cap is not filled

            // all unstaked funds can go away immediately
            self.fund_transf_module().swap_unstaked_to_deferred_payment(&mut total_unstaked, || false);
            require!(total_unstaked == 0, "error swapping unstaked to deferred payment");

            // we need to see how much of the delegation cap remains unfilled
            let total_active = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
            let total_delegation_cap = self.settings().get_total_delegation_cap();
            let mut fillable_active_stake = &total_delegation_cap - &total_active;
            
            // swap waiting -> active, but no more than fillable
            // no need to worry about rewards here, because there aren't any
            let _ = self.fund_transf_module().swap_waiting_to_active(&mut fillable_active_stake, || false);
            if fillable_active_stake == 0 {
                // this happens only when waiting was enough to fill the delegation cap
                self.settings().set_bootstrap_mode(false);
            }
            Ok(())
        } else {
            // regular scenario
            // exactly the same amount is swapped from waiting -> active, as from unstaked -> deferred payment
            let swappable = core::cmp::min(&total_waiting, &total_unstaked);
            if *swappable == 0 {
                return Ok(())
            }

            // swap unStaked -> deferred payment
            let mut unstaked_swap_remaining = swappable.clone();
            self.fund_transf_module().swap_unstaked_to_deferred_payment(&mut unstaked_swap_remaining, || false);
            require!(unstaked_swap_remaining == 0, "error swapping unstaked to deferred payment");

            // swap waiting -> active (also compute rewards)
            self.swap_waiting_to_active_compute_rewards(&swappable)
        }
    }

    /// Swaps waiting stake to active within given limits,
    /// and also computes rewards for all affected users before performing the swap itself.
    fn swap_waiting_to_active_compute_rewards(&self, swappable: &BigUint) -> SCResult<()> {
        // dry run of swap, to get the affected users
        let (affected_users, remaining) = 
            self.fund_transf_module().get_affected_users_of_swap_waiting_to_active(swappable, || false);
        require!(remaining == 0, "error swapping waiting to active");

        // compute rewards for all affected users
        self.rewards().compute_one_user_reward(OWNER_USER_ID);
        for user_id in affected_users.iter() {
            let user_id_nz = non_zero_usize!(*user_id, "bad user_id");
            self.rewards().compute_one_user_reward(user_id_nz);
        }

        // actual swap of waiting to active
        let mut remaining = swappable.clone();
        let _ = self.fund_transf_module().swap_waiting_to_active(&mut remaining, || false);
        require!(remaining == 0, "error swapping waiting to active");

        Ok(())
    }

    /// Delegate stake to the smart contract. 
    /// Stake is initially inactive, so does it not produce rewards.
    #[payable]
    #[endpoint(stake)]
    fn stake_endpoint(&self, #[payment] payment: BigUint) -> SCResult<()> {
        require!(self.pause().not_paused(), "contract paused");

        require!(payment >= self.settings().get_minimum_stake(),
            "cannot stake less than minimum stake");

        require!(!self.reset_checkpoints().is_global_op_in_progress(),
            "staking is temporarily paused as checkpoint is reset");

        self.process_stake(payment)
    }

    // WITHDRAW INACTIVE

    #[endpoint(withdrawInactiveStake)]
    fn withdraw_inactive_stake(&self, amount: BigUint) -> SCResult<()> {
        require!(self.pause().not_paused(), "contract paused");

        if amount == 0 {
            return Ok(());
        }

        let caller = self.get_caller();
        let user_id = self.user_data().get_user_id(&caller);
        require!(user_id > 0,
            "only delegators can withdraw inactive stake");

        let mut amount_to_withdraw = amount.clone();
        sc_try!(self.fund_transf_module().liquidate_free_stake(user_id, &mut amount_to_withdraw));
        require!(amount_to_withdraw == 0,
            "cannot withdraw more than inactive stake");

        // send stake to delegator
        self.send_tx(&caller, &amount, "delegation withdraw inactive stake");

        self.events().unstake_event(&caller, &amount);
        Ok(())
    }

    /// Mostly invariant: modifyTotalDelegationCap can violate this rule.
    fn validate_user_minimum_stake(&self, user_id: usize) -> SCResult<()> {
        let waiting = self.fund_view_module().get_user_stake_of_type(user_id, FundType::Waiting);
        let active = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
        let relevant_stake = &waiting + &active;

        require!(relevant_stake == 0 || relevant_stake >= self.settings().get_minimum_stake(),
            "cannot have waiting + active stake less than minimum stake");
        Ok(())
    }

    /// Invariant: should never return error.
    #[view(validateOwnerStakeShare)]
    fn validate_owner_stake_share(&self) -> SCResult<()> {
        // owner total stake / contract total stake < owner_min_stake_share / 10000
        // reordered to avoid divisions
        require!(
            self.fund_view_module().get_user_stake_of_type(OWNER_USER_ID.get(), FundType::Active) * 
            BigUint::from(PERCENTAGE_DENOMINATOR)
            >=
            self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active) *
            self.settings().get_owner_min_stake_share(),
            "owner doesn't have enough stake in the contract");
        Ok(())
    }

    /// Invariant: should never return error.
    #[view(validateDelegationCapInvariant)]
    fn validate_delegation_cap_invariant(&self) -> SCResult<()> {
        let total_active = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
        let total_unstaked = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked);
        let total_delegation_cap = self.settings().get_total_delegation_cap();
        
        if self.settings().is_bootstrap_mode() {
            require!(&total_active + &total_unstaked < total_delegation_cap,
                "delegation cap invariant violated");
        } else {
            require!(&total_active + &total_unstaked == total_delegation_cap,
                "delegation cap invariant violated");
        }
        
        Ok(())
    }
    
}
