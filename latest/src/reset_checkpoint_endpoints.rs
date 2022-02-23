use crate::reset_checkpoint_types::{
    ComputeAllRewardsData, GlobalOpCheckpoint, ModifyDelegationCapStep,
    ModifyTotalDelegationCapData,
};
use crate::settings::{OWNER_USER_ID, PERCENTAGE_DENOMINATOR};
use core::cmp::Ordering;
use user_fund_storage::fund_view_module::USER_STAKE_TOTALS_ID;
use user_fund_storage::types::FundType;

elrond_wasm::imports!();

pub const STOP_AT_GASLIMIT: u64 = 100_000_000;

#[elrond_wasm::derive::module]
pub trait ResetCheckpointsModule:
    crate::rewards_state::RewardStateModule
    + crate::reset_checkpoint_state::ResetCheckpointStateModule
    + elrond_wasm_modules::features::FeaturesModule
    + user_fund_storage::user_data::UserDataModule
    + user_fund_storage::fund_module::FundModule
    + user_fund_storage::fund_view_module::FundViewModule
    + user_fund_storage::fund_transf_module::FundTransformationsModule
    + crate::settings::SettingsModule
{
    /// Continues executing any interrupted operation.
    /// Returns true if still out of gas, false if computation completed.
    #[endpoint(continueGlobalOperation)]
    fn continue_global_operation_endpoint(&self) -> OperationCompletionStatus {
        self.check_feature_on(b"continueGlobalOperation", true);

        let orc = self.global_op_checkpoint().get();
        self.continue_global_operation(orc)
    }

    fn continue_global_operation(
        &self,
        mut orc: Box<GlobalOpCheckpoint<Self::Api>>,
    ) -> OperationCompletionStatus {
        let mut status = OperationCompletionStatus::Completed;
        while matches!(status, OperationCompletionStatus::Completed) && !orc.is_none() {
            let (new_status, new_orc) = self.continue_global_operation_step(orc);
            status = new_status;
            orc = new_orc;
        }

        self.global_op_checkpoint().set(&orc);
        status
    }

    fn continue_global_operation_step(
        &self,
        orc: Box<GlobalOpCheckpoint<Self::Api>>,
    ) -> (
        OperationCompletionStatus,
        Box<GlobalOpCheckpoint<Self::Api>>,
    ) {
        match *orc {
            GlobalOpCheckpoint::None => (OperationCompletionStatus::Completed, orc),
            GlobalOpCheckpoint::ModifyTotalDelegationCap(mdcap_data) => {
                self.continue_modify_total_delegation_cap_step(mdcap_data)
            }
            GlobalOpCheckpoint::ChangeServiceFee {
                new_service_fee,
                compute_rewards_data,
            } => {
                if let Some(more_computation) = self.compute_all_rewards(compute_rewards_data) {
                    (
                        OperationCompletionStatus::InterruptedBeforeOutOfGas,
                        Box::new(GlobalOpCheckpoint::ChangeServiceFee {
                            new_service_fee,
                            compute_rewards_data: more_computation,
                        }),
                    )
                } else {
                    // finish
                    self.set_service_fee(new_service_fee);
                    (
                        OperationCompletionStatus::Completed,
                        Box::new(GlobalOpCheckpoint::None),
                    )
                }
            }
        }
    }

    fn continue_modify_total_delegation_cap_step(
        &self,
        mut mdcap_data: ModifyTotalDelegationCapData<Self::Api>,
    ) -> (
        OperationCompletionStatus,
        Box<GlobalOpCheckpoint<Self::Api>>,
    ) {
        match mdcap_data.step {
            ModifyDelegationCapStep::ComputeAllRewards(car_data) => {
                if let Some(more_computation) = self.compute_all_rewards(car_data) {
                    mdcap_data.step = ModifyDelegationCapStep::ComputeAllRewards(more_computation);
                    (
                        OperationCompletionStatus::InterruptedBeforeOutOfGas,
                        Box::new(GlobalOpCheckpoint::ModifyTotalDelegationCap(mdcap_data)),
                    )
                } else {
                    mdcap_data.step = ModifyDelegationCapStep::SwapWaitingToActive;
                    (
                        OperationCompletionStatus::Completed,
                        Box::new(GlobalOpCheckpoint::ModifyTotalDelegationCap(mdcap_data)),
                    )
                }
            }
            ModifyDelegationCapStep::SwapWaitingToActive => {
                let _ = self.swap_waiting_to_active(
                    &mut mdcap_data.remaining_swap_waiting_to_active, // decreases this field directly
                    || self.blockchain().get_gas_left() < STOP_AT_GASLIMIT,
                );
                if mdcap_data.remaining_swap_waiting_to_active > 0 {
                    (
                        OperationCompletionStatus::InterruptedBeforeOutOfGas,
                        Box::new(GlobalOpCheckpoint::ModifyTotalDelegationCap(mdcap_data)),
                    )
                } else {
                    mdcap_data.step = ModifyDelegationCapStep::SwapUnstakedToDeferredPayment;
                    (
                        OperationCompletionStatus::Completed,
                        Box::new(GlobalOpCheckpoint::ModifyTotalDelegationCap(mdcap_data)),
                    )
                }
            }
            ModifyDelegationCapStep::SwapUnstakedToDeferredPayment => {
                self.swap_unstaked_to_deferred_payment(
                    &mut mdcap_data.remaining_swap_unstaked_to_def_p, // decreases this field directly
                    || self.blockchain().get_gas_left() < STOP_AT_GASLIMIT,
                );
                if mdcap_data.remaining_swap_unstaked_to_def_p > 0 {
                    (
                        OperationCompletionStatus::InterruptedBeforeOutOfGas,
                        Box::new(GlobalOpCheckpoint::ModifyTotalDelegationCap(mdcap_data)),
                    )
                } else {
                    mdcap_data.step = ModifyDelegationCapStep::SwapActiveToDeferredPayment;
                    (
                        OperationCompletionStatus::Completed,
                        Box::new(GlobalOpCheckpoint::ModifyTotalDelegationCap(mdcap_data)),
                    )
                }
            }
            ModifyDelegationCapStep::SwapActiveToDeferredPayment => {
                self.swap_active_to_deferred_payment(
                    &mut mdcap_data.remaining_swap_active_to_def_p, // decreases this field directly
                    || self.blockchain().get_gas_left() < STOP_AT_GASLIMIT,
                );
                if mdcap_data.remaining_swap_active_to_def_p > 0 {
                    (
                        OperationCompletionStatus::InterruptedBeforeOutOfGas,
                        Box::new(GlobalOpCheckpoint::ModifyTotalDelegationCap(mdcap_data)),
                    )
                } else {
                    // finish
                    self.set_total_delegation_cap(mdcap_data.new_delegation_cap);
                    (
                        OperationCompletionStatus::Completed,
                        Box::new(GlobalOpCheckpoint::None),
                    )
                }
            }
        }
    }

    /// When there is a change of the base cap from which the rewards are computed,
    /// the checkpoints must be reset for all the delegators.
    /// This process might be longer then one block - reaching the gaslimit
    /// thus will do it by saving where it left before reaching out of gas.
    /// No change in the delegators total cap is allowed before all the checkpoints are recalculated.
    ///
    /// Returns something if there is more computing to be done.
    fn compute_all_rewards(
        &self,
        mut data: ComputeAllRewardsData<Self::Api>,
    ) -> Option<ComputeAllRewardsData<Self::Api>> {
        // if more rewards arrived since computation started,
        // it must be restarted from scratch
        let curr_rewards_checkpoint = self.get_total_cumulated_rewards();
        if data.rewards_checkpoint != curr_rewards_checkpoint {
            data.last_id = 0;
            data.sum_unclaimed = BigUint::zero();
            data.rewards_checkpoint = curr_rewards_checkpoint;
        }

        let num_nodes = self.get_num_users();

        while data.last_id < num_nodes {
            if self.blockchain().get_gas_left() < STOP_AT_GASLIMIT {
                return Some(data);
            }

            let current_user_id = non_zero_usize_from_n_plus_1(data.last_id);
            let user_data = self.load_updated_user_rewards(current_user_id);
            self.store_user_reward_data(current_user_id, &user_data);
            data.sum_unclaimed += user_data.unclaimed_rewards;
            data.last_id = current_user_id.get();
        }

        // divisions are inexact so a small remainder can remain after distributing rewards
        // give it to the owner, to keep things clear
        let remainder =
            &self.get_total_cumulated_rewards() - &data.sum_unclaimed - self.get_sent_rewards();
        if remainder > 0 {
            let mut node_unclaimed = self.get_user_rew_unclaimed(OWNER_USER_ID);
            node_unclaimed += &remainder;
            self.set_user_rew_unclaimed(OWNER_USER_ID, &node_unclaimed);
        }

        None
    }

    /// Total delegation cap can be modified by owner only.
    /// It will recalculate and set the checkpoint for all the delegators
    #[only_owner]
    #[endpoint(modifyTotalDelegationCap)]
    fn modify_total_delegation_cap(&self, new_total_cap: BigUint) -> OperationCompletionStatus {
        require!(
            !self.is_global_op_in_progress(),
            "cannot modify total delegation cap when last is in progress"
        );

        let total_waiting = self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Waiting);
        let total_active = self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
        let total_unstaked = self.get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked);

        let previous_total_cap: BigUint;
        let max_available = &(&total_active + &total_waiting) + &total_unstaked;
        if self.is_bootstrap_mode() {
            if new_total_cap > max_available {
                // we remain in bootstrap mode
                // and so nothing else to be done here:
                // compute all rewards not necessary - no rewards yet
                // swap not necessary - there cannot be any waiting or unstaked funds
                self.set_total_delegation_cap(new_total_cap);
                return OperationCompletionStatus::Completed;
            } else {
                // bootstrap mode is over
                // no rewards to compute, but
                // swap might be necessary
                self.set_bootstrap_mode(false);

                // This scenario is equivalent to performing 2 operations:
                // 1. drop from the previous delegation cap to max_amount - nothing happens to the funds.
                // 2. drop from max_amount to the new_total_cap. This might involve some swaps.
                // From here on, only step 2. needs to be performed, so we set the previous cap to what te max_amount was.
                previous_total_cap = max_available;
            }
        } else {
            // if no longer in bootstrap mode, total delegation cap can never exceed the max available
            require!(
                new_total_cap <= max_available,
                "new delegation cap must be less or equal to total active + waiting"
            );

            // The old total cap is simply the one from storage.
            previous_total_cap = self.get_total_delegation_cap();
        }

        let orc = match new_total_cap.cmp(&previous_total_cap) {
            Ordering::Equal => {
                // nothing changes
                return OperationCompletionStatus::Completed;
            }
            Ordering::Greater => {
                // cap increases
                require!(
                    total_unstaked == 0u32,
                    "no unstaked funds should be present when increasing delegation cap"
                );

                let swap_amount = &new_total_cap - &previous_total_cap;
                Box::new(GlobalOpCheckpoint::ModifyTotalDelegationCap(
                    ModifyTotalDelegationCapData {
                        new_delegation_cap: new_total_cap,
                        remaining_swap_waiting_to_active: swap_amount,
                        remaining_swap_active_to_def_p: BigUint::zero(),
                        remaining_swap_unstaked_to_def_p: BigUint::zero(),
                        step: ModifyDelegationCapStep::ComputeAllRewards(
                            ComputeAllRewardsData::new(self.get_total_cumulated_rewards()),
                        ),
                    },
                ))
            }
            Ordering::Less => {
                // cap decreases
                let swap_amount = &previous_total_cap - &new_total_cap;
                require!(
                    swap_amount <= self.total_unprotected(),
                    "not enough funds in contract to pay those who are forced unstaked"
                );

                let swap_unstaked_to_def_p: BigUint;
                let swap_active_to_def_p: BigUint;
                if total_unstaked >= swap_amount {
                    // only unstaked -> deferred payment will happen
                    swap_active_to_def_p = BigUint::zero();
                    swap_unstaked_to_def_p = swap_amount;
                } else {
                    // first unstaked -> deferred payment happens, then active -> deferred payment
                    swap_active_to_def_p = &swap_amount - &total_unstaked;
                    swap_unstaked_to_def_p = total_unstaked;
                }

                Box::new(GlobalOpCheckpoint::ModifyTotalDelegationCap(
                    ModifyTotalDelegationCapData {
                        new_delegation_cap: new_total_cap,
                        remaining_swap_waiting_to_active: BigUint::zero(),
                        remaining_swap_active_to_def_p: swap_active_to_def_p,
                        remaining_swap_unstaked_to_def_p: swap_unstaked_to_def_p,
                        step: ModifyDelegationCapStep::ComputeAllRewards(
                            ComputeAllRewardsData::new(self.get_total_cumulated_rewards()),
                        ),
                    },
                ))
            }
        };

        self.continue_global_operation(orc)
    }

    /// The stake per node can be changed by the owner.
    /// It does not get set in the contructor, so the owner has to manually set it after the contract is deployed.
    #[endpoint(setServiceFee)]
    fn set_service_fee_endpoint(&self, service_fee_per_10000: usize) -> OperationCompletionStatus {
        require!(
            service_fee_per_10000 <= PERCENTAGE_DENOMINATOR,
            "service fee out of range"
        );

        require!(
            !self.is_global_op_in_progress(),
            "global checkpoint is in progress"
        );

        let new_service_fee = BigUint::from(service_fee_per_10000);
        if self.get_service_fee() == new_service_fee {
            return OperationCompletionStatus::Completed;
        }

        if self.is_bootstrap_mode() {
            // no rewards to compute
            // change service fee directly
            self.set_service_fee(new_service_fee);
            OperationCompletionStatus::Completed
        } else {
            // start compute all rewards
            self.continue_global_operation(Box::new(GlobalOpCheckpoint::ChangeServiceFee {
                new_service_fee,
                compute_rewards_data: ComputeAllRewardsData::new(
                    self.get_total_cumulated_rewards(),
                ),
            }))
        }
    }
}
