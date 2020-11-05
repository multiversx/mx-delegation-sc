use elrond_wasm::elrond_codec::*;
use elrond_wasm::BigUintApi;

/// Functions return this as status, if operation was completed or not.
#[derive(PartialEq, Debug)]
pub enum GlobalOperationStatus {
    Done,
    StoppedBeforeOutOfGas
}

impl GlobalOperationStatus {
    fn to_u64(&self) -> u64 {
        match self {
            GlobalOperationStatus::Done => 0,
            GlobalOperationStatus::StoppedBeforeOutOfGas => 1,
        } 
    }

    pub fn is_done(&self) -> bool {
        *self == GlobalOperationStatus::Done
    }
}

impl TopEncode for GlobalOperationStatus {
    #[inline]
    fn top_encode<O: TopEncodeOutput>(&self, output: O) -> Result<(), EncodeError> {
        self.to_u64().top_encode(output)
    }

    #[inline]
    fn top_encode_or_exit<O: TopEncodeOutput, ExitCtx: Clone>(&self, output: O, c: ExitCtx, exit: fn(ExitCtx, EncodeError) -> !) {
        self.to_u64().top_encode_or_exit(output, c, exit);
    }
}

/// Models any computation that can pause itself when it runs out of gas and continue in another block.
#[derive(PartialEq, Debug)]
pub enum GlobalOperationCheckpoint<BigUint:BigUintApi> {
    None,
    ModifyTotalDelegationCap(ModifyTotalDelegationCapData<BigUint>),
    ChangeServiceFee{
        new_service_fee: BigUint,
        compute_rewards_data: ComputeAllRewardsData<BigUint>,
    },
}

impl<BigUint:BigUintApi> GlobalOperationCheckpoint<BigUint> {
    #[inline]
    pub fn is_none(&self) -> bool {
        *self == GlobalOperationCheckpoint::<BigUint>::None
    }

    #[inline]
    pub fn is_zero_value(&self) -> bool {
        self.is_none()
    }

    pub fn zero_value() -> Self {
        GlobalOperationCheckpoint::None
    }
}

impl<BigUint:BigUintApi> NestedEncode for GlobalOperationCheckpoint<BigUint> {
    fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
        match self {
            GlobalOperationCheckpoint::None => {
                dest.push_byte(0);
            },
            GlobalOperationCheckpoint::ModifyTotalDelegationCap(data) => {
                dest.push_byte(1);
                data.dep_encode(dest)?;
            },
            GlobalOperationCheckpoint::ChangeServiceFee{
                new_service_fee,
                compute_rewards_data,
            } => {
                dest.push_byte(2);
                new_service_fee.dep_encode(dest)?;
                compute_rewards_data.dep_encode(dest)?;
            },
        }
        Ok(())
    }

    fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(&self, dest: &mut O, c: ExitCtx, exit: fn(ExitCtx, EncodeError) -> !) {
        match self {
            GlobalOperationCheckpoint::None => {
                dest.push_byte(0);
            },
            GlobalOperationCheckpoint::ModifyTotalDelegationCap(data) => {
                dest.push_byte(1);
                data.dep_encode_or_exit(dest, c.clone(), exit);
            },
            GlobalOperationCheckpoint::ChangeServiceFee{
                new_service_fee,
                compute_rewards_data,
            } => {
                dest.push_byte(2);
                new_service_fee.dep_encode_or_exit(dest, c.clone(), exit);
                compute_rewards_data.dep_encode_or_exit(dest, c.clone(), exit);
            },
        }
    }
}

impl<BigUint:BigUintApi> TopEncode for GlobalOperationCheckpoint<BigUint> {
    #[inline]
    fn top_encode<O: TopEncodeOutput>(&self, output: O) -> Result<(), EncodeError> {
        // delete storage when the balance reaches 0
        // also require links to have been reset (this check is not strictly necessary, but improves safety)
        if self.is_zero_value() {
            output.set_slice_u8(&[]);
            Ok(())
        } else {
            top_encode_from_nested(self, output)
        }
    }

    #[inline]
    fn top_encode_or_exit<O: TopEncodeOutput, ExitCtx: Clone>(&self, output: O, c: ExitCtx, exit: fn(ExitCtx, EncodeError) -> !) {
        // delete storage when the balance reaches 0
        // also require links to have been reset (this check is not strictly necessary, but improves safety)
        if self.is_zero_value() {
            output.set_slice_u8(&[]);
        } else {
            top_encode_from_nested_or_exit(self, output, c, exit);
        }
    }
}

impl<BigUint:BigUintApi> NestedDecode for GlobalOperationCheckpoint<BigUint> {
    fn dep_decode<I: NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
        let discriminant = input.read_byte()?;
        match discriminant {
            0 => Ok(GlobalOperationCheckpoint::None),
            1 => Ok(GlobalOperationCheckpoint::ModifyTotalDelegationCap(
                ModifyTotalDelegationCapData::dep_decode(input)?
            )),
            2 => Ok(GlobalOperationCheckpoint::ChangeServiceFee{
                new_service_fee: BigUint::dep_decode(input)?,
                compute_rewards_data: ComputeAllRewardsData::dep_decode(input)?,
            }),
            _ => Err(DecodeError::INVALID_VALUE),
        }
    }

    fn dep_decode_or_exit<I: NestedDecodeInput, ExitCtx: Clone>(input: &mut I, c: ExitCtx, exit: fn(ExitCtx, DecodeError) -> !) -> Self {
        let discriminant = input.read_byte_or_exit(c.clone(), exit);
        match discriminant {
            0 => GlobalOperationCheckpoint::None,
            1 => GlobalOperationCheckpoint::ModifyTotalDelegationCap(
                ModifyTotalDelegationCapData::dep_decode_or_exit(input, c.clone(), exit),
            ),
            2 => GlobalOperationCheckpoint::ChangeServiceFee{
                new_service_fee: BigUint::dep_decode_or_exit(input, c.clone(), exit),
                compute_rewards_data: ComputeAllRewardsData::dep_decode_or_exit(input, c.clone(), exit),
            },
            _ => exit(c, DecodeError::INVALID_VALUE),
        }
    }
}

impl<BigUint:BigUintApi> TopDecode for GlobalOperationCheckpoint<BigUint> {
    fn top_decode<I: TopDecodeInput>(input: I) -> Result<Self, DecodeError> {
        if input.byte_len() == 0 {
            // does not exist in storage 
            Ok(GlobalOperationCheckpoint::zero_value())
        } else {
            top_decode_from_nested(input)
        }
    }

    fn top_decode_or_exit<I: TopDecodeInput, ExitCtx: Clone>(input: I, c: ExitCtx, exit: fn(ExitCtx, DecodeError) -> !) -> Self {
        if input.byte_len() == 0 {
            // does not exist in storage 
            GlobalOperationCheckpoint::zero_value()
        } else {
            top_decode_from_nested_or_exit(input, c, exit)
        }
    }
}

/// Contains data needed to be persisted while performing a change in the total delegation cap.
#[derive(PartialEq, Debug)]
pub struct ModifyTotalDelegationCapData<BigUint:BigUintApi> {
    pub new_delegation_cap: BigUint,
    pub remaining_swap_waiting_to_active: BigUint,
    pub remaining_swap_active_to_def_p: BigUint,
    pub remaining_swap_unstaked_to_def_p: BigUint,
    pub step: ModifyDelegationCapStep<BigUint>,
}

impl<BigUint:BigUintApi> NestedEncode for ModifyTotalDelegationCapData<BigUint> {
    fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.new_delegation_cap.dep_encode(dest)?;
        self.remaining_swap_waiting_to_active.dep_encode(dest)?;
        self.remaining_swap_active_to_def_p.dep_encode(dest)?;
        self.remaining_swap_unstaked_to_def_p.dep_encode(dest)?;
        self.step.dep_encode(dest)?;
        Ok(())
    }

    fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(&self, dest: &mut O, c: ExitCtx, exit: fn(ExitCtx, EncodeError) -> !) {
        self.new_delegation_cap.dep_encode_or_exit(dest, c.clone(), exit);
        self.remaining_swap_waiting_to_active.dep_encode_or_exit(dest, c.clone(), exit);
        self.remaining_swap_active_to_def_p.dep_encode_or_exit(dest, c.clone(), exit);
        self.remaining_swap_unstaked_to_def_p.dep_encode_or_exit(dest, c.clone(), exit);
        self.step.dep_encode_or_exit(dest, c.clone(), exit);
    }
}

impl<BigUint:BigUintApi> NestedDecode for ModifyTotalDelegationCapData<BigUint> {
    fn dep_decode<I: NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(ModifyTotalDelegationCapData{
            new_delegation_cap: BigUint::dep_decode(input)?,
            remaining_swap_waiting_to_active: BigUint::dep_decode(input)?,
            remaining_swap_active_to_def_p: BigUint::dep_decode(input)?,
            remaining_swap_unstaked_to_def_p: BigUint::dep_decode(input)?,
            step: ModifyDelegationCapStep::dep_decode(input)?,
        })
    }

    fn dep_decode_or_exit<I: NestedDecodeInput, ExitCtx: Clone>(input: &mut I, c: ExitCtx, exit: fn(ExitCtx, DecodeError) -> !) -> Self {
        ModifyTotalDelegationCapData{
            new_delegation_cap: BigUint::dep_decode_or_exit(input, c.clone(), exit),
            remaining_swap_waiting_to_active: BigUint::dep_decode_or_exit(input, c.clone(), exit),
            remaining_swap_active_to_def_p: BigUint::dep_decode_or_exit(input, c.clone(), exit),
            remaining_swap_unstaked_to_def_p: BigUint::dep_decode_or_exit(input, c.clone(), exit),
            step: ModifyDelegationCapStep::dep_decode_or_exit(input, c.clone(), exit),
        }
    }
}

/// Models the steps that need to be executed when modifying the total delegation cap.
#[derive(PartialEq, Debug)]
pub enum ModifyDelegationCapStep<BigUint:BigUintApi> {
    ComputeAllRewards(ComputeAllRewardsData<BigUint>),
    SwapWaitingToActive,
    SwapUnstakedToDeferredPayment,
    SwapActiveToDeferredPayment,
}

impl<BigUint:BigUintApi> NestedEncode for ModifyDelegationCapStep<BigUint> {
    fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
        match self {
            ModifyDelegationCapStep::ComputeAllRewards(data) => {
                dest.push_byte(0);
                data.dep_encode(dest)?;
            },
            ModifyDelegationCapStep::SwapWaitingToActive => { dest.push_byte(1); },
            ModifyDelegationCapStep::SwapUnstakedToDeferredPayment => { dest.push_byte(2); },
            ModifyDelegationCapStep::SwapActiveToDeferredPayment => { dest.push_byte(3); },
        }
        Ok(())
    }

    fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(&self, dest: &mut O, c: ExitCtx, exit: fn(ExitCtx, EncodeError) -> !) {
        match self {
            ModifyDelegationCapStep::ComputeAllRewards(data) => {
                dest.push_byte(0);
                data.dep_encode_or_exit(dest, c.clone(), exit);
            },
            ModifyDelegationCapStep::SwapWaitingToActive => { dest.push_byte(1); },
            ModifyDelegationCapStep::SwapUnstakedToDeferredPayment => { dest.push_byte(2); },
            ModifyDelegationCapStep::SwapActiveToDeferredPayment => { dest.push_byte(3); },
        }
    }
}

impl<BigUint:BigUintApi> NestedDecode for ModifyDelegationCapStep<BigUint> {
    fn dep_decode<I: NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
        let discriminant = input.read_byte()?;
        match discriminant {
            0 => Ok(ModifyDelegationCapStep::ComputeAllRewards(
                ComputeAllRewardsData::dep_decode(input)?)),
            1 => Ok(ModifyDelegationCapStep::SwapWaitingToActive),
            2 => Ok(ModifyDelegationCapStep::SwapUnstakedToDeferredPayment),
            3 => Ok(ModifyDelegationCapStep::SwapActiveToDeferredPayment),
            _ => Err(DecodeError::INVALID_VALUE),
        }
    }

    fn dep_decode_or_exit<I: NestedDecodeInput, ExitCtx: Clone>(input: &mut I, c: ExitCtx, exit: fn(ExitCtx, DecodeError) -> !) -> Self {
        let discriminant = input.read_byte_or_exit(c.clone(), exit);
        match discriminant {
            0 => ModifyDelegationCapStep::ComputeAllRewards(
                ComputeAllRewardsData::dep_decode_or_exit(input, c, exit)),
            1 => ModifyDelegationCapStep::SwapWaitingToActive,
            2 => ModifyDelegationCapStep::SwapUnstakedToDeferredPayment,
            3 => ModifyDelegationCapStep::SwapActiveToDeferredPayment,
            _ => exit(c, DecodeError::INVALID_VALUE),
        }
    }
}

/// Models the interrupted state of compute_all_rewards.
#[derive(PartialEq, Debug)]
pub struct ComputeAllRewardsData<BigUint:BigUintApi> {
    pub last_id:              usize,
    pub sum_unclaimed:        BigUint,
    pub rewards_checkpoint:   BigUint,
}

impl<BigUint:BigUintApi> ComputeAllRewardsData<BigUint> {
    pub fn new(rewards_checkpoint: BigUint) -> ComputeAllRewardsData<BigUint> {
        ComputeAllRewardsData {
            last_id: 0,
            sum_unclaimed: BigUint::zero(),
            rewards_checkpoint,
        }
    }
}

impl<BigUint:BigUintApi> NestedEncode for ComputeAllRewardsData<BigUint> {
    fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.last_id.dep_encode(dest)?;
        self.sum_unclaimed.dep_encode(dest)?;
        self.rewards_checkpoint.dep_encode(dest)?;
        Ok(())
    }

    fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(&self, dest: &mut O, c: ExitCtx, exit: fn(ExitCtx, EncodeError) -> !) {
        self.last_id.dep_encode_or_exit(dest, c.clone(), exit);
        self.sum_unclaimed.dep_encode_or_exit(dest, c.clone(), exit);
        self.rewards_checkpoint.dep_encode_or_exit(dest, c.clone(), exit);
    }
}

impl<BigUint:BigUintApi> NestedDecode for ComputeAllRewardsData<BigUint> {
    fn dep_decode<I: NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(ComputeAllRewardsData{
            last_id: usize::dep_decode(input)?,
            sum_unclaimed: BigUint::dep_decode(input)?,
            rewards_checkpoint: BigUint::dep_decode(input)?,
        })
    }

    fn dep_decode_or_exit<I: NestedDecodeInput, ExitCtx: Clone>(input: &mut I, c: ExitCtx, exit: fn(ExitCtx, DecodeError) -> !) -> Self {
        ComputeAllRewardsData{
            last_id: usize::dep_decode_or_exit(input, c.clone(), exit),
            sum_unclaimed: BigUint::dep_decode_or_exit(input, c.clone(), exit),
            rewards_checkpoint: BigUint::dep_decode_or_exit(input, c.clone(), exit),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elrond_wasm::elrond_codec::test_util::*;
    use elrond_wasm_debug::*;

    fn check_global_operation_checkpoint_codec(goc: GlobalOperationCheckpoint<RustBigUint>) {
        let top_encoded = check_top_encode(&goc);
        let top_decoded = check_top_decode::<GlobalOperationCheckpoint::<RustBigUint>>(&top_encoded[..]);
        assert_eq!(top_decoded, goc);

        let dep_encoded = check_dep_encode(&goc);
        let dep_decoded = check_dep_decode::<GlobalOperationCheckpoint::<RustBigUint>>(&dep_encoded[..]);
        assert_eq!(dep_decoded, goc);
    }

    #[test]
    fn test_global_operation_checkpoint() {
        check_global_operation_checkpoint_codec(
            GlobalOperationCheckpoint::None
        );

        check_global_operation_checkpoint_codec(
            GlobalOperationCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                new_delegation_cap: 104u32.into(),
                remaining_swap_waiting_to_active: 105u32.into(),
                remaining_swap_active_to_def_p: 106u32.into(),
                remaining_swap_unstaked_to_def_p: 107u32.into(),
                step: ModifyDelegationCapStep::ComputeAllRewards(ComputeAllRewardsData{
                    last_id:              108,
                    sum_unclaimed:        109u32.into(),
                    rewards_checkpoint:   110u32.into(),
                }),
            })
        );

        check_global_operation_checkpoint_codec(
            GlobalOperationCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                new_delegation_cap: 104u32.into(),
                remaining_swap_waiting_to_active: 105u32.into(),
                remaining_swap_active_to_def_p: 106u32.into(),
                remaining_swap_unstaked_to_def_p: 107u32.into(),
                step: ModifyDelegationCapStep::SwapWaitingToActive,
            })
        );

        check_global_operation_checkpoint_codec(
            GlobalOperationCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                new_delegation_cap: 104u32.into(),
                remaining_swap_waiting_to_active: 105u32.into(),
                remaining_swap_active_to_def_p: 106u32.into(),
                remaining_swap_unstaked_to_def_p: 107u32.into(),
                step: ModifyDelegationCapStep::SwapActiveToDeferredPayment,
            })
        );
        
        check_global_operation_checkpoint_codec(
            GlobalOperationCheckpoint::ModifyTotalDelegationCap(ModifyTotalDelegationCapData{
                new_delegation_cap: 104u32.into(),
                remaining_swap_waiting_to_active: 105u32.into(),
                remaining_swap_active_to_def_p: 106u32.into(),
                remaining_swap_unstaked_to_def_p: 107u32.into(),
                step: ModifyDelegationCapStep::SwapUnstakedToDeferredPayment,
            })
        );

        check_global_operation_checkpoint_codec(
            GlobalOperationCheckpoint::ChangeServiceFee{
                new_service_fee: 190u32.into(),
                compute_rewards_data: ComputeAllRewardsData{
                    last_id:              108,
                    sum_unclaimed:        109u32.into(),
                    rewards_checkpoint:   110u32.into(),
                }
            }
        );
    }
}
