use elrond_wasm::api::BigUintApi;

elrond_wasm::derive_imports!();

/// Models any computation that can pause itself when it runs out of gas and continue in another block.
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Debug)]
pub enum GlobalOpCheckpoint<BigUint: BigUintApi> {
    None,
    ModifyTotalDelegationCap(ModifyTotalDelegationCapData<BigUint>),
    ChangeServiceFee {
        new_service_fee: BigUint,
        compute_rewards_data: ComputeAllRewardsData<BigUint>,
    },
}

impl<BigUint: BigUintApi> GlobalOpCheckpoint<BigUint> {
    #[inline]
    pub fn is_none(&self) -> bool {
        *self == GlobalOpCheckpoint::<BigUint>::None
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
pub struct ModifyTotalDelegationCapData<BigUint: BigUintApi> {
    pub new_delegation_cap: BigUint,
    pub remaining_swap_waiting_to_active: BigUint,
    pub remaining_swap_active_to_def_p: BigUint,
    pub remaining_swap_unstaked_to_def_p: BigUint,
    pub step: ModifyDelegationCapStep<BigUint>,
}

/// Models the steps that need to be executed when modifying the total delegation cap.
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Debug)]
pub enum ModifyDelegationCapStep<BigUint: BigUintApi> {
    ComputeAllRewards(ComputeAllRewardsData<BigUint>),
    SwapWaitingToActive,
    SwapUnstakedToDeferredPayment,
    SwapActiveToDeferredPayment,
}

/// Models the interrupted state of compute_all_rewards.
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Debug)]
pub struct ComputeAllRewardsData<BigUint: BigUintApi> {
    pub last_id: usize,
    pub sum_unclaimed: BigUint,
    pub rewards_checkpoint: BigUint,
}

impl<BigUint: BigUintApi> ComputeAllRewardsData<BigUint> {
    pub fn new(rewards_checkpoint: BigUint) -> ComputeAllRewardsData<BigUint> {
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
    use elrond_wasm_debug::api::RustBigUint;

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
