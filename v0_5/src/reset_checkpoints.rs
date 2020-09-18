
use crate::rewards::*;
use crate::settings::*;
use user_fund_storage::user_data::*;
use user_fund_storage::fund_transf_module::*;
use user_fund_storage::fund_view_module::*;
use user_fund_storage::types::*;
use crate::reset_checkpoint_types::*;
use elrond_wasm_module_features::*;
use elrond_wasm_module_pause::*;
use core::cmp::Ordering;

imports!();

pub const STOP_AT_GASLIMIT: i64 = 1000000;

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

    #[module(PauseModuleImpl)]
    fn pause(&self) -> PauseModuleImpl<T, BigInt, BigUint>;

    #[module(FeaturesModuleImpl)]
    fn features_module(&self) -> FeaturesModuleImpl<T, BigInt, BigUint>;

    #[view(getGlobalOperationCheckpoint)]
    #[storage_get("global_op_checkpoint")]
    fn get_global_op_checkpoint(&self) -> GlobalOperationCheckpoint<BigUint>;

    #[storage_set("global_op_checkpoint")]
    fn set_global_op_checkpoint(&self, orc: &GlobalOperationCheckpoint<BigUint>);

    #[view(isGlobalOperationInProgress)]
    fn is_global_op_in_progress(&self) -> bool {
        // TODO: make this pattern into an attribute just like storage_get/storage_set in elrond_wasm
        // something like storage_is_empty
        self.storage_load_len(&b"global_op_checkpoint"[..]) > 0
    }

    /// Continues executing any interrupted operation.
    /// Returns true if still out of gas, false if computation completed.
    #[endpoint(continueGlobalOperation)]
    fn continue_global_operation_endpoint(&self) -> SCResult<GlobalOperationStatus> {
        feature_guard!(self.features_module(), b"continueGlobalOperation", true);

        let orc = self.get_global_op_checkpoint();
        self.continue_global_operation(orc)
    }

    fn continue_global_operation(&self, mut orc: GlobalOperationCheckpoint<BigUint>) -> SCResult<GlobalOperationStatus> {
        let mut status = GlobalOperationStatus::Done;
        while status.is_done() && !orc.is_none() {
            let (new_status, new_orc) = self.continue_global_operation_step(orc);
            status = new_status;
            orc = new_orc;
        }

        self.set_global_op_checkpoint(&orc); 
        Ok(status)
    }

    fn continue_global_operation_step(&self, 
            orc: GlobalOperationCheckpoint<BigUint>)
            -> (GlobalOperationStatus, GlobalOperationCheckpoint<BigUint>) {

        match orc {
            GlobalOperationCheckpoint::None => (GlobalOperationStatus::Done, orc),
            GlobalOperationCheckpoint::ModifyTotalDelegationCap(mdcap_data) =>
                self.continue_modify_total_delegation_cap_step(mdcap_data),
            GlobalOperationCheckpoint::ChangeServiceFee{
                new_service_fee,
                compute_rewards_data,
            } => {
                if let Some(more_computation) = self.compute_all_rewards(compute_rewards_data) {
                    (GlobalOperationStatus::StoppedBeforeOutOfGas, GlobalOperationCheckpoint::ChangeServiceFee{
                        new_service_fee,
                        compute_rewards_data: more_computation,
                    })
                } else {
                    // finish
                    self.settings().set_service_fee(new_service_fee);
                    (GlobalOperationStatus::Done, GlobalOperationCheckpoint::None)
                }
            },
        }
    }

    fn continue_modify_total_delegation_cap_step(&self, 
        mut mdcap_data: ModifyTotalDelegationCapData<BigUint>)
        -> (GlobalOperationStatus, GlobalOperationCheckpoint<BigUint>) {
        
        match mdcap_data.step {
            ModifyDelegationCapStep::ComputeAllRewards(car_data) => {
                if let Some(more_computation) = self.compute_all_rewards(car_data) {
                    mdcap_data.step = ModifyDelegationCapStep::ComputeAllRewards(more_computation);
                    (GlobalOperationStatus::StoppedBeforeOutOfGas, GlobalOperationCheckpoint::ModifyTotalDelegationCap(mdcap_data))
                } else {
                    mdcap_data.step = ModifyDelegationCapStep::SwapWaitingToActive;
                    (GlobalOperationStatus::Done, GlobalOperationCheckpoint::ModifyTotalDelegationCap(mdcap_data))
                }
            },
            ModifyDelegationCapStep::SwapWaitingToActive => {
                let _ = self.fund_transf_module().swap_waiting_to_active(
                    &mut mdcap_data.remaining_swap_waiting_to_active, // decreases this field directly
                    || self.get_gas_left() < STOP_AT_GASLIMIT
                );
                if mdcap_data.remaining_swap_waiting_to_active > 0 {
                    (GlobalOperationStatus::StoppedBeforeOutOfGas, GlobalOperationCheckpoint::ModifyTotalDelegationCap(mdcap_data))
                } else {
                    mdcap_data.step = ModifyDelegationCapStep::SwapUnstakedToDeferredPayment;
                    (GlobalOperationStatus::Done, GlobalOperationCheckpoint::ModifyTotalDelegationCap(mdcap_data))
                }
            },
            ModifyDelegationCapStep::SwapUnstakedToDeferredPayment => {
                self.fund_transf_module().swap_unstaked_to_deferred_payment(
                    &mut mdcap_data.remaining_swap_unstaked_to_def_p, // decreases this field directly
                    || self.get_gas_left() < STOP_AT_GASLIMIT
                );
                if mdcap_data.remaining_swap_unstaked_to_def_p > 0 {
                    (GlobalOperationStatus::StoppedBeforeOutOfGas, GlobalOperationCheckpoint::ModifyTotalDelegationCap(mdcap_data))
                } else {
                    mdcap_data.step = ModifyDelegationCapStep::SwapActiveToDeferredPayment;
                    (GlobalOperationStatus::Done, GlobalOperationCheckpoint::ModifyTotalDelegationCap(mdcap_data))
                }
            },
            ModifyDelegationCapStep::SwapActiveToDeferredPayment => {
                self.fund_transf_module().swap_active_to_deferred_payment(
                    &mut mdcap_data.remaining_swap_active_to_def_p, // decreases this field directly
                    || self.get_gas_left() < STOP_AT_GASLIMIT
                );
                if mdcap_data.remaining_swap_active_to_def_p > 0 {
                    (GlobalOperationStatus::StoppedBeforeOutOfGas, GlobalOperationCheckpoint::ModifyTotalDelegationCap(mdcap_data))
                } else {
                    // finish
                    self.settings().set_total_delegation_cap(mdcap_data.new_delegation_cap);
                    (GlobalOperationStatus::Done, GlobalOperationCheckpoint::None)
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
        // if more rewards arrived since computation started,
        // it must be restarted from scratch
        let curr_rewards_checkpoint = self.rewards().get_total_cumulated_rewards();
        if data.rewards_checkpoint != curr_rewards_checkpoint {
            data.last_id = 0;
            data.sum_unclaimed = BigUint::zero();
            data.rewards_checkpoint = curr_rewards_checkpoint;
        }

        let num_nodes = self.user_data().get_num_users();

        while data.last_id < num_nodes {
            if self.get_gas_left() < STOP_AT_GASLIMIT {
                return Some(data);
            }

            let current_user_id = non_zero_usize_from_n_plus_1(data.last_id);
            let user_data = self.rewards().load_updated_user_rewards(current_user_id);
            self.rewards().store_user_reward_data(current_user_id, &user_data);
            data.sum_unclaimed += user_data.unclaimed_rewards;
            data.last_id = current_user_id.get();
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
    fn modify_total_delegation_cap(&self, new_total_cap: BigUint) -> SCResult<GlobalOperationStatus> {
        only_owner!(self,
            "only owner allowed to modify delegation cap");

        require!(!self.is_global_op_in_progress(),
            "cannot modify total delegation cap when last is in progress");

        let curr_delegation_cap = self.settings().get_total_delegation_cap();
        let total_waiting = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Waiting);
        let total_active = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::Active);
        let total_unstaked = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, FundType::UnStaked);

        let max_available = &(&total_active + &total_waiting) + &total_unstaked;
        if self.settings().is_bootstrap_mode() {
            if new_total_cap > max_available {
                // we remain in bootstrap mode
                // and so nothing else to be done here:
                // compute all rewards not necessary - no rewards yet
                // swap not necessary - there cannot be any waiting or unstaked funds
                return Ok(GlobalOperationStatus::Done);
            } else {
                // bootstrap mode is over
                // no rewards to compute, but
                // swap might be necessary
                self.settings().set_bootstrap_mode(false);
            }
        } else {
            // if no longer in bootstrap mode, total delegation cap can never exceed the max available
            require!(new_total_cap <= max_available,
                "new delegation cap must be less or equal to total active + waiting");
        }

        let orc = match new_total_cap.cmp(&curr_delegation_cap) {
            Ordering::Equal => { // nothing changes
                return Ok(GlobalOperationStatus::Done)
            },
            Ordering::Greater => { // cap increases
                require!(total_unstaked == 0,
                    "no unstaked funds should be present when increasing delegation cap");

                let swap_amount = &new_total_cap - &curr_delegation_cap;
                GlobalOperationCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                    new_delegation_cap: new_total_cap,
                    remaining_swap_waiting_to_active: swap_amount,
                    remaining_swap_active_to_def_p: BigUint::zero(),
                    remaining_swap_unstaked_to_def_p: BigUint::zero(),
                    step: ModifyDelegationCapStep::ComputeAllRewards(ComputeAllRewardsData::new(self.rewards().get_total_cumulated_rewards())),
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
                
                GlobalOperationCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                    new_delegation_cap: new_total_cap,
                    remaining_swap_waiting_to_active: BigUint::zero(),
                    remaining_swap_active_to_def_p: swap_active_to_def_p,
                    remaining_swap_unstaked_to_def_p: swap_unstaked_to_def_p,
                    step: ModifyDelegationCapStep::ComputeAllRewards(ComputeAllRewardsData::new(self.rewards().get_total_cumulated_rewards())),
                })
            }
        };

        self.continue_global_operation(orc)
    }
}
