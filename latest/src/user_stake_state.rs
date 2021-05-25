use super::elrond_wasm_module_pause::*;
use super::user_fund_storage::fund_transf_module::*;
use super::user_fund_storage::fund_view_module::*;
use super::user_fund_storage::types::*;
use super::user_fund_storage::user_data::*;
use crate::events::*;
use crate::reset_checkpoint_endpoints::*;
use crate::rewards_state::*;
use crate::settings::*;

use core::cmp::Ordering;
use core::num::NonZeroUsize;

elrond_wasm::imports!();

/// Contains endpoints for staking/withdrawing stake.
#[elrond_wasm_derive::module]
pub trait UserStakeStateModule:
    crate::settings::SettingsModule
    + crate::user_fund_storage::user_data::UserDataModule
    + crate::user_fund_storage::fund_module::FundModule
    + crate::user_fund_storage::fund_view_module::FundViewModule
    + crate::user_fund_storage::fund_transf_module::FundTransformationsModule
    + crate::rewards_state::RewardStateModule
    + crate::events::EventsModule
{
    // #[module(UserDataModuleImpl)]
    // fn user_data(&self) -> UserDataModuleImpl<T, BigInt, Self::BigUint>;

    // #[module(FundTransformationsModuleImpl)]
    // fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, Self::BigUint>;

    // #[module(FundViewModuleImpl)]
    // fn fund_view_module(&self) -> FundViewModuleImpl<T, BigInt, Self::BigUint>;

    // #[module(PauseModuleImpl)]
    // fn pause(&self) -> PauseModuleImpl<T, BigInt, Self::BigUint>;

    // #[module(RewardsModuleImpl)]
    // fn rewards(&self) -> RewardsModuleImpl<T, BigInt, Self::BigUint>;

    // #[module(ResetCheckpointsModuleImpl)]
    // fn reset_checkpoints(&self) -> ResetCheckpointsModuleImpl<T, BigInt, Self::BigUint>;

    // #[module(SettingsModuleImpl)]
    // fn settings(&self) -> SettingsModuleImpl<T, BigInt, Self::BigUint>;

    // #[module(EventsModuleImpl)]
    // fn events(&self) -> EventsModuleImpl<T, BigInt, Self::BigUint>;

    fn process_stake(&self, payment: Self::BigUint) -> SCResult<()> {
        // get user id or create user
        // we use user id as an intermediate identifier between user address and data,
        // because we might at some point need to iterate over all user data
        let caller = self.blockchain().get_caller();
        let user_id = self.get_or_create_user(&caller);

        // log staking event
        self.stake_event(&caller, &payment);

        // create stake funds
        self.create_waiting(user_id, payment);

        // check invariant
        self.validate_delegation_cap_invariant()?;

        // move funds around
        self.use_waiting_to_replace_unstaked()
    }

    /// The contract can be either overstaked (waiting > 0) or understaked (unstaked > 0).
    /// Cannot have both, since waiting should "cancel out" the unstaked.
    /// This operation does this. It takes min(waiting, unstaked) and converts this amount
    /// from waiting to active and from unstaked to deferred payment.
    /// Note that this operation preserves the invariant that active + unstaked == delegation_cap.
    fn use_waiting_to_replace_unstaked(&self) -> SCResult<()> {
        let total_waiting = self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Waiting);
        let mut total_unstaked =
            self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked);

        if self.is_bootstrap_mode() {
            // bootstrap mode!
            // the total delegation cap is not filled

            // all unstaked funds can go away immediately
            self.swap_unstaked_to_deferred_payment(&mut total_unstaked, || false);
            require!(
                total_unstaked == 0,
                "error swapping unstaked to deferred payment"
            );

            // we need to see how much of the delegation cap remains unfilled
            let total_active = self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
            let total_delegation_cap = self.get_total_delegation_cap();
            let mut fillable_active_stake = &total_delegation_cap - &total_active;

            // swap waiting -> active, but no more than fillable
            // no need to worry about rewards here, because there aren't any
            let _ = self.swap_waiting_to_active(&mut fillable_active_stake, || false);
            if fillable_active_stake == 0 {
                // this happens only when waiting was enough to fill the delegation cap
                self.set_bootstrap_mode(false);
            }
            Ok(())
        } else {
            // regular scenario
            // exactly the same amount is swapped from waiting -> active, as from unstaked -> deferred payment
            let swappable = core::cmp::min(&total_waiting, &total_unstaked);
            if *swappable == 0 {
                return Ok(());
            }

            // swap unStaked -> deferred payment
            let mut unstaked_swap_remaining = swappable.clone();
            self.swap_unstaked_to_deferred_payment(&mut unstaked_swap_remaining, || false);
            require!(
                unstaked_swap_remaining == 0,
                "error swapping unstaked to deferred payment"
            );

            // swap waiting -> active (also compute rewards)
            self.swap_waiting_to_active_compute_rewards(&swappable)
        }
    }

    /// Swaps waiting stake to active within given limits,
    /// and also computes rewards for all affected users before performing the swap itself.
    fn swap_waiting_to_active_compute_rewards(&self, swappable: &Self::BigUint) -> SCResult<()> {
        // dry run of swap, to get the affected users
        let (affected_users, remaining) =
            self.get_affected_users_of_swap_waiting_to_active(swappable, || false);
        require!(remaining == 0, "error swapping waiting to active");

        // compute rewards for all affected users
        self.compute_one_user_reward(OWNER_USER_ID);
        for user_id in affected_users.iter() {
            let user_id_nz = non_zero_usize!(*user_id, "bad user_id");
            self.compute_one_user_reward(user_id_nz);
        }

        // actual swap of waiting to active
        let mut remaining = swappable.clone();
        let _ = self.swap_waiting_to_active(&mut remaining, || false);
        require!(remaining == 0, "error swapping waiting to active");

        Ok(())
    }

    /// Mostly invariant: modifyTotalDelegationCap can violate this rule.
    fn validate_user_minimum_stake(&self, user_id: usize) -> SCResult<()> {
        let waiting = self.get_user_stake_of_type(user_id, FundType::Waiting);
        let active = self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
        let relevant_stake = &waiting + &active;

        require!(
            relevant_stake == 0 || relevant_stake >= self.get_minimum_stake(),
            "cannot have waiting + active stake less than minimum stake"
        );
        Ok(())
    }

    /// Invariant: should never return error.
    #[view(validateOwnerStakeShare)]
    fn validate_owner_stake_share(&self) -> SCResult<()> {
        // owner total stake / contract total stake < owner_min_stake_share / 10000
        // reordered to avoid divisions
        require!(
            self.get_user_stake_of_type(OWNER_USER_ID.get(), FundType::Active)
                * Self::BigUint::from(PERCENTAGE_DENOMINATOR)
                >= self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active)
                    * self.get_owner_min_stake_share(),
            "owner doesn't have enough stake in the contract"
        );
        Ok(())
    }

    fn validate_unstake_amount(&self, user_id: usize, amount: &Self::BigUint) -> SCResult<()> {
        let max_unstake = self.get_user_stake_of_type(user_id, FundType::Waiting)
            + self.get_user_stake_of_type(user_id, FundType::Active);
        match amount.cmp(&max_unstake) {
            Ordering::Greater => {
                sc_error!("cannot unstake more than the user waiting + active stake")
            }
            Ordering::Equal => Ok(()),
            Ordering::Less => {
                require!(
                    *amount >= self.get_minimum_stake(),
                    "cannot unstake less than minimum stake"
                );
                Ok(())
            }
        }
    }

    /// Invariant: should never return error.
    #[view(validateDelegationCapInvariant)]
    fn validate_delegation_cap_invariant(&self) -> SCResult<()> {
        let total_active = self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
        let total_unstaked = self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked);
        let total_delegation_cap = self.get_total_delegation_cap();

        if self.is_bootstrap_mode() {
            require!(
                &total_active + &total_unstaked < total_delegation_cap,
                "delegation cap invariant violated"
            );
        } else {
            require!(
                &total_active + &total_unstaked == total_delegation_cap,
                "delegation cap invariant violated"
            );
        }

        Ok(())
    }
}
