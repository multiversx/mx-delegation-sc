use multiversx_sc::{api::ManagedTypeApi, types::BigUint};

multiversx_sc::derive_imports!();

/// Models any computation that can pause itself when it runs out of gas and continue in another block.
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug)]
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
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug)]
pub struct ModifyTotalDelegationCapData<M: ManagedTypeApi> {
    pub new_delegation_cap: BigUint<M>,
    pub remaining_swap_waiting_to_active: BigUint<M>,
    pub remaining_swap_active_to_def_p: BigUint<M>,
    pub remaining_swap_unstaked_to_def_p: BigUint<M>,
    pub step: ModifyDelegationCapStep<M>,
}

/// Models the steps that need to be executed when modifying the total delegation cap.
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug)]
pub enum ModifyDelegationCapStep<M: ManagedTypeApi> {
    ComputeAllRewards(ComputeAllRewardsData<M>),
    SwapWaitingToActive,
    SwapUnstakedToDeferredPayment,
    SwapActiveToDeferredPayment,
}

/// Models the interrupted state of compute_all_rewards.
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug)]
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
