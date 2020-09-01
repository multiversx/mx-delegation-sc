
use crate::rewards::*;
use crate::settings::*;
use user_fund_storage::user_data::*;
use user_fund_storage::fund_transf_module::*;
use user_fund_storage::fund_view_module::*;
use user_fund_storage::types::*;
use crate::reset_checkpoint_types::*;
use core::cmp::Ordering;

imports!();

pub static STOP_AT_GASLIMIT: i64 = 1000000;

pub const COMPUTATION_DONE: bool = false;
pub const OUT_OF_GAS: bool = true;

#[elrond_wasm_derive::module(ResetCheckpointsModuleImpl)]
pub trait ResetCheckpointsModule {
    
    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(FundViewModuleImpl)]
    fn fund_view_module(&self) -> FundViewModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;


    #[view(getInterruptedComputation)]
    #[storage_get("interrupted_computation")]
    fn get_interrupted_computation(&self) -> OngoingResetCheckpoint<BigUint>;

    #[storage_set("interrupted_computation")]
    fn set_interrupted_computation(&self, ec: &OngoingResetCheckpoint<BigUint>);

    #[view(isInterruptedComputation)]
    fn is_interrupted_computation(&self) -> bool {
        // TODO: make this pattern into an attribute just like storage_get/storage_set in elrond_wasm
        // something like storage_is_empty
        self.storage_load_len(&b"interrupted_computation"[..]) > 0
    }

    /// Continues executing any interrupted operation.
    /// Returns true if still out of gas, false if computation completed.
    #[endpoint(continueComputation)]
    fn continue_computation_endpoint(&self) -> SCResult<bool> {
        let ec = self.get_interrupted_computation();
        self.perform_extended_computation(ec)
    }

    fn perform_extended_computation(&self, mut ec: OngoingResetCheckpoint<BigUint>) -> SCResult<bool> {
        let mut out_of_gas = false;
        while !out_of_gas && !ec.is_none() {
            let result = self.perform_interrupted_computation_step(ec);
            out_of_gas = result.0;
            ec = result.1;
        }

        self.set_interrupted_computation(&ec); 
        Ok(out_of_gas)
    }

    fn perform_interrupted_computation_step(&self, ec: OngoingResetCheckpoint<BigUint>) -> (bool, OngoingResetCheckpoint<BigUint>) {
        match ec {
            OngoingResetCheckpoint::None => (false, ec),
            OngoingResetCheckpoint::ModifyTotalDelegationCap(mdcap_data) => {
                match mdcap_data.step {
                    ModifyDelegationCapStep::ComputeAllRewards(car_data) => {
                        if let Some(more_computation) = self.compute_all_rewards(car_data) {
                            (OUT_OF_GAS, OngoingResetCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                                step: ModifyDelegationCapStep::ComputeAllRewards(more_computation),
                                ..mdcap_data
                            }))
                        } else {
                            (COMPUTATION_DONE, OngoingResetCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                                step: ModifyDelegationCapStep::SwapUnstakedToDeferredPayment,
                                ..mdcap_data
                            }))
                        }
                    },
                    ModifyDelegationCapStep::SwapWaitingToActive => {
                        let (_, remaining) = self.fund_transf_module().swap_waiting_to_active(
                            &mdcap_data.remaining_swap_waiting_to_active,
                            || self.get_gas_left() < STOP_AT_GASLIMIT
                        );
                        if remaining > 0 {
                            (OUT_OF_GAS, OngoingResetCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                                remaining_swap_waiting_to_active: remaining,
                                ..mdcap_data
                            }))
                        } else {
                            (COMPUTATION_DONE, OngoingResetCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                                remaining_swap_waiting_to_active: BigUint::zero(),
                                step: ModifyDelegationCapStep::SwapUnstakedToDeferredPayment,
                                ..mdcap_data
                            }))
                        }
                    },
                    ModifyDelegationCapStep::SwapUnstakedToDeferredPayment => {
                        let remaining = self.fund_transf_module().swap_unstaked_to_deferred_payment(
                            &mdcap_data.remaining_swap_unstaked_to_def_p,
                            || self.get_gas_left() < STOP_AT_GASLIMIT
                        );
                        if remaining > 0 {
                            (OUT_OF_GAS, OngoingResetCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                                remaining_swap_unstaked_to_def_p: remaining,
                                ..mdcap_data
                            }))
                        } else {
                            (COMPUTATION_DONE, OngoingResetCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                                remaining_swap_unstaked_to_def_p: BigUint::zero(),
                                step: ModifyDelegationCapStep::SwapActiveToDeferredPayment,
                                ..mdcap_data
                            }))
                        }
                    },
                    ModifyDelegationCapStep::SwapActiveToDeferredPayment => {
                        let remaining = self.fund_transf_module().swap_active_to_deferred_payment(
                            &mdcap_data.remaining_swap_active_to_def_p,
                            || self.get_gas_left() < STOP_AT_GASLIMIT
                        );
                        if remaining > 0 {
                            (OUT_OF_GAS, OngoingResetCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                                remaining_swap_active_to_def_p: remaining,
                                ..mdcap_data
                            }))
                        } else {
                            // finish
                            self.settings().set_total_delegation_cap(mdcap_data.new_delegation_cap);
                            (COMPUTATION_DONE, OngoingResetCheckpoint::None)
                        }
                    },
                }
            },
            OngoingResetCheckpoint::ChangeServiceFee{
                new_service_fee,
                compute_rewards_data,
            } => {
                if let Some(more_computation) = self.compute_all_rewards(compute_rewards_data) {
                    (OUT_OF_GAS, OngoingResetCheckpoint::ChangeServiceFee{
                        new_service_fee,
                        compute_rewards_data: more_computation,
                    })
                } else {
                    // finish
                    self.settings().set_service_fee(new_service_fee);
                    (COMPUTATION_DONE, OngoingResetCheckpoint::None)
                }
            },
        }
    }


    /// When there is a change of the base cap from which the rewards are computed,
    /// the checkpoints must be reset for all the delegators.
    /// This process might be longer then one block - reaching the gaslimit
    /// thus will do it by saving where it left before reaching out of gas.
    /// No change in the delegators total cap is allowed before all the checkpoints are recalculated.
    /// 
    /// Returns something if there is more computing to be done.
    fn compute_all_rewards(&self, mut data: ComputeAllRewardsData<BigUint>) -> Option<ComputeAllRewardsData<BigUint>> {
        // if epoch changed, computation must be started from scratch
        // TODO: base this on reward checkpoint instead of epoch to fix edge case
        let curr_epoch = self.get_block_epoch();
        if data.epoch != curr_epoch {
            data.last_id = 0;
            data.sum_unclaimed = BigUint::zero();
            data.epoch = curr_epoch;
        }

        let num_nodes = self.user_data().get_num_users();

        while data.last_id <= num_nodes {
            if self.get_gas_left() < STOP_AT_GASLIMIT {
                return Some(data);
            }

            let current_user_id = data.last_id + 1;
            let user_data = self.rewards().load_updated_user_rewards(current_user_id);
            self.rewards().store_user_reward_data(current_user_id, &user_data);
            data.sum_unclaimed += user_data.unclaimed_rewards;
            data.last_id = current_user_id;
        }

        // divisions are inexact so a small remainder can remain after distributing rewards
        // give it to the owner, to keep things clear
        let remainder = &self.rewards().get_total_cumulated_rewards() - &data.sum_unclaimed - self.rewards().get_sent_rewards();
        if remainder > 0 {
            let mut node_unclaimed = self.rewards().get_user_rew_unclaimed(OWNER_USER_ID);
            node_unclaimed += &remainder;
            self.rewards().set_user_rew_unclaimed(OWNER_USER_ID, &node_unclaimed);
        }

        None
    }

    /// Total delegation cap can be modified by owner only.
    /// It will recalculate and set the checkpoint for all the delegators
    #[endpoint(modifyTotalDelegationCap)]
    fn modify_total_delegation_cap(&self, new_total_cap: BigUint) -> SCResult<bool> {
        require!(self.settings().owner_called(),
            "only owner allowed to modify delegation cap");

        require!(!self.is_interrupted_computation(),
            "cannot modify total delegation cap when last is in progress");

        let curr_delegation_cap = self.settings().get_total_delegation_cap();
        let total_waiting = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Waiting);
        let total_active = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
        let total_unstaked = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked);

        let max_available = &total_active + &total_waiting;
        require!(new_total_cap <= max_available,
            "new delegation cap must be less or equal to total active + waiting");

        let ec = match new_total_cap.cmp(&curr_delegation_cap) {
            Ordering::Equal => { // nothing changes
                return Ok(COMPUTATION_DONE)
            },
            Ordering::Greater => { // cap increases
                require!(total_unstaked == 0,
                    "no unstaked funds should be present when increasing delegation cap");

                let swap_amount = &new_total_cap - &curr_delegation_cap;
                OngoingResetCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                    new_delegation_cap: new_total_cap,
                    remaining_swap_waiting_to_active: swap_amount,
                    remaining_swap_active_to_def_p: BigUint::zero(),
                    remaining_swap_unstaked_to_def_p: BigUint::zero(),
                    step: ModifyDelegationCapStep::ComputeAllRewards(ComputeAllRewardsData::new(self.get_block_epoch())),
                })
            },
            Ordering::Less => { // cap decreases
                let swap_amount = &curr_delegation_cap - &new_total_cap;
                require!(swap_amount <= self.rewards().total_unprotected(),
                    "not enough funds in contract to pay those who are forced unstaked");
                
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
                
                OngoingResetCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                    new_delegation_cap: new_total_cap,
                    remaining_swap_waiting_to_active: BigUint::zero(),
                    remaining_swap_active_to_def_p: swap_active_to_def_p,
                    remaining_swap_unstaked_to_def_p: swap_unstaked_to_def_p,
                    step: ModifyDelegationCapStep::ComputeAllRewards(ComputeAllRewardsData::new(self.get_block_epoch())),
                })
            }
        };

        self.perform_extended_computation(ec)
    }
}
