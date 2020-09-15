use elrond_wasm::elrond_codec::*;
use elrond_wasm::BigUintApi;
use elrond_wasm::Vec;

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
}

impl<BigUint:BigUintApi> Encode for GlobalOperationCheckpoint<BigUint> {
    fn using_top_encoded<F: FnOnce(&[u8])>(&self, f: F) -> Result<(), EncodeError> {
        // None clears the storage
        if let GlobalOperationCheckpoint::None = self {
            f(&[]);
        } else {
            let mut result: Vec<u8> = Vec::new();
            self.dep_encode_to(&mut result)?;
            f(result.as_slice());
        }
        Ok(())
    }

	fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        match self {
            GlobalOperationCheckpoint::None => {
                dest.push_byte(0);
            },
            GlobalOperationCheckpoint::ModifyTotalDelegationCap(data) => {
                dest.push_byte(1);
                data.dep_encode_to(dest)?;
            },
            GlobalOperationCheckpoint::ChangeServiceFee{
                new_service_fee,
                compute_rewards_data,
            } => {
                dest.push_byte(2);
                new_service_fee.dep_encode_to(dest)?;
                compute_rewards_data.dep_encode_to(dest)?;
            },
        }
        Ok(())
	}
}

impl<BigUint:BigUintApi> Decode for GlobalOperationCheckpoint<BigUint> {
    fn top_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        if input.remaining_len() == 0 {
            // does not exist in storage 
            Ok(GlobalOperationCheckpoint::None)
        } else {
            let result = Self::dep_decode(input)?;
            if input.remaining_len() > 0 {
                return Err(DecodeError::InputTooLong);
            }
            Ok(result)
        }
    }

    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
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
            _ => Err(DecodeError::InvalidValue),
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

impl<BigUint:BigUintApi> Encode for ModifyTotalDelegationCapData<BigUint> {
	fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.new_delegation_cap.dep_encode_to(dest)?;
        self.remaining_swap_waiting_to_active.dep_encode_to(dest)?;
        self.remaining_swap_active_to_def_p.dep_encode_to(dest)?;
        self.remaining_swap_unstaked_to_def_p.dep_encode_to(dest)?;
        self.step.dep_encode_to(dest)?;
        Ok(())
	}
}

impl<BigUint:BigUintApi> Decode for ModifyTotalDelegationCapData<BigUint> {
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(ModifyTotalDelegationCapData{
            new_delegation_cap: BigUint::dep_decode(input)?,
            remaining_swap_waiting_to_active: BigUint::dep_decode(input)?,
            remaining_swap_active_to_def_p: BigUint::dep_decode(input)?,
            remaining_swap_unstaked_to_def_p: BigUint::dep_decode(input)?,
            step: ModifyDelegationCapStep::dep_decode(input)?,
        })
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

impl<BigUint:BigUintApi> Encode for ModifyDelegationCapStep<BigUint> {
    fn dep_encode_to<O: Output>(&self, dest: &mut O)  -> Result<(), EncodeError> {
        match self {
            ModifyDelegationCapStep::ComputeAllRewards(data) => {
                dest.push_byte(0);
                data.dep_encode_to(dest)?;
            },
            ModifyDelegationCapStep::SwapWaitingToActive => { dest.push_byte(1); },
            ModifyDelegationCapStep::SwapUnstakedToDeferredPayment => { dest.push_byte(2); },
            ModifyDelegationCapStep::SwapActiveToDeferredPayment => { dest.push_byte(3); },
        }
        Ok(())
    }
}

impl<BigUint:BigUintApi> Decode for ModifyDelegationCapStep<BigUint> {
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        let discriminant = input.read_byte()?;
        match discriminant {
            0 => Ok(ModifyDelegationCapStep::ComputeAllRewards(ComputeAllRewardsData::dep_decode(input)?)),
            1 => Ok(ModifyDelegationCapStep::SwapWaitingToActive),
            2 => Ok(ModifyDelegationCapStep::SwapUnstakedToDeferredPayment),
            3 => Ok(ModifyDelegationCapStep::SwapActiveToDeferredPayment),
            _ => Err(DecodeError::InvalidValue),
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

impl<BigUint:BigUintApi> Encode for ComputeAllRewardsData<BigUint> {
	fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.last_id.dep_encode_to(dest)?;
        self.sum_unclaimed.dep_encode_to(dest)?;
        self.rewards_checkpoint.dep_encode_to(dest)?;
        Ok(())
	}
}

impl<BigUint:BigUintApi> Decode for ComputeAllRewardsData<BigUint> {
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(ComputeAllRewardsData{
            last_id: usize::dep_decode(input)?,
            sum_unclaimed: BigUint::dep_decode(input)?,
            rewards_checkpoint: BigUint::dep_decode(input)?,
        })
    }
}


