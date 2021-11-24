use elrond_wasm::{api::ManagedTypeApi, types::BigUint};


elrond_wasm::derive_imports!();

/// Models any computation that can pause itself when it runs out of gas and continue in another block.
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Debug)]
pub enum GlobalOpCheckpoint<M: ManagedTypeApi> {
    None,
    ModifyTotalDelegationCap(ModifyTotalDelegationCapData<M>),
    ChangeServiceFee {
        new_service_fee: BigUint<M>,
        compute_rewards_data: ComputeAllRewardsData<M>,
    },
}

impl<M: ManagedTypeApi> GlobalOpCheckpoint<M> {
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, GlobalOpCheckpoint::None)
    }

    #[inline]
    pub fn is_zero_value(&self) -> bool {
        self.is_none()
    }

    pub fn zero_value() -> Self {
        GlobalOpCheckpoint::None
    }
}

/// Contains data needed to be persisted while performing a change in the total delegation cap.
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Debug)]
pub struct ModifyTotalDelegationCapData<M: ManagedTypeApi> {
    pub new_delegation_cap: BigUint<M>,
    pub remaining_swap_waiting_to_active: BigUint<M>,
    pub remaining_swap_active_to_def_p: BigUint<M>,
    pub remaining_swap_unstaked_to_def_p: BigUint<M>,
    pub step: ModifyDelegationCapStep<M>,
}

/// Models the steps that need to be executed when modifying the total delegation cap.
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Debug)]
pub enum ModifyDelegationCapStep<M: ManagedTypeApi> {
    ComputeAllRewards(ComputeAllRewardsData<M>),
    SwapWaitingToActive,
    SwapUnstakedToDeferredPayment,
    SwapActiveToDeferredPayment,
}

/// Models the interrupted state of compute_all_rewards.
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Debug)]
pub struct ComputeAllRewardsData<M: ManagedTypeApi> {
    pub last_id: usize,
    pub sum_unclaimed: BigUint<M>,
    pub rewards_checkpoint: BigUint<M>,
}

impl<M: ManagedTypeApi> ComputeAllRewardsData<M> {
    pub fn new(rewards_checkpoint: BigUint<M>) -> ComputeAllRewardsData<M> {
        ComputeAllRewardsData {
            last_id: 0,
            sum_unclaimed: BigUint::zero(),
            rewards_checkpoint,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elrond_wasm::elrond_codec::test_util::*;

    fn check_global_operation_checkpoint_codec(goc: GlobalOpCheckpoint<RustBigUint>) {
        let top_encoded = check_top_encode(&goc);
        let top_decoded = check_top_decode::<GlobalOpCheckpoint<RustBigUint>>(&top_encoded[..]);
        assert_eq!(top_decoded, goc);

        let dep_encoded = check_dep_encode(&goc);
        let dep_decoded = check_dep_decode::<GlobalOpCheckpoint<RustBigUint>>(&dep_encoded[..]);
        assert_eq!(dep_decoded, goc);
    }

    #[test]
    fn test_global_operation_checkpoint() {
        check_global_operation_checkpoint_codec(GlobalOpCheckpoint::None);

        check_global_operation_checkpoint_codec(GlobalOpCheckpoint::ModifyTotalDelegationCap(
            ModifyTotalDelegationCapData {
                new_delegation_cap: 104u32.into(),
                remaining_swap_waiting_to_active: 105u32.into(),
                remaining_swap_active_to_def_p: 106u32.into(),
                remaining_swap_unstaked_to_def_p: 107u32.into(),
                step: ModifyDelegationCapStep::ComputeAllRewards(ComputeAllRewardsData {
                    last_id: 108,
                    sum_unclaimed: 109u32.into(),
                    rewards_checkpoint: 110u32.into(),
                }),
            },
        ));

        check_global_operation_checkpoint_codec(GlobalOpCheckpoint::ModifyTotalDelegationCap(
            ModifyTotalDelegationCapData {
                new_delegation_cap: 104u32.into(),
                remaining_swap_waiting_to_active: 105u32.into(),
                remaining_swap_active_to_def_p: 106u32.into(),
                remaining_swap_unstaked_to_def_p: 107u32.into(),
                step: ModifyDelegationCapStep::SwapWaitingToActive,
            },
        ));

        check_global_operation_checkpoint_codec(GlobalOpCheckpoint::ModifyTotalDelegationCap(
            ModifyTotalDelegationCapData {
                new_delegation_cap: 104u32.into(),
                remaining_swap_waiting_to_active: 105u32.into(),
                remaining_swap_active_to_def_p: 106u32.into(),
                remaining_swap_unstaked_to_def_p: 107u32.into(),
                step: ModifyDelegationCapStep::SwapActiveToDeferredPayment,
            },
        ));

        check_global_operation_checkpoint_codec(GlobalOpCheckpoint::ModifyTotalDelegationCap(
            ModifyTotalDelegationCapData {
                new_delegation_cap: 104u32.into(),
                remaining_swap_waiting_to_active: 105u32.into(),
                remaining_swap_active_to_def_p: 106u32.into(),
                remaining_swap_unstaked_to_def_p: 107u32.into(),
                step: ModifyDelegationCapStep::SwapUnstakedToDeferredPayment,
            },
        ));

        check_global_operation_checkpoint_codec(GlobalOpCheckpoint::ChangeServiceFee {
            new_service_fee: 190u32.into(),
            compute_rewards_data: ComputeAllRewardsData {
                last_id: 108,
                sum_unclaimed: 109u32.into(),
                rewards_checkpoint: 110u32.into(),
            },
        });
    }
}
