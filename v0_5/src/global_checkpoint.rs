use elrond_wasm::elrond_codec::*;
use elrond_wasm::BigUintApi;
use elrond_wasm::Vec;
use user_fund_storage::types::*;

#[derive(Clone, PartialEq)]
pub struct GlobalCheckpoint<BigUint:BigUintApi> {
    pub total_delegation_cap: BigUint,
    pub sum_unclaimed:        BigUint,
    pub total_to_swap:        BigUint,
    pub last_id:              usize,
    pub epoch:                u64,
}

impl<BigUint:BigUintApi> Encode for GlobalCheckpoint<BigUint> {
    fn dep_encode_to<O: Output>(&self, dest: &mut O)  -> Result<(), EncodeError> {
        self.total_delegation_cap.dep_encode_to(dest)?;
        self.sum_unclaimed.dep_encode_to(dest)?;
        self.total_to_swap.dep_encode_to(dest)?;
        self.last_id.dep_encode_to(dest)?;
        self.epoch.dep_encode_to(dest)?;
        Ok(())
    }
}

impl<BigUint:BigUintApi> Decode for GlobalCheckpoint<BigUint> {
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(GlobalCheckpoint{
            total_delegation_cap: BigUint::dep_decode(input)?,
            sum_unclaimed:        BigUint::dep_decode(input)?,
            total_to_swap:        BigUint::dep_decode(input)?,
            last_id:              usize::dep_decode(input)?,
            epoch:                u64::dep_decode(input)?,
        })
    }
}

#[derive(Clone, PartialEq)]
pub struct SwapCheckpoint<BigUint:BigUintApi> {
    pub initial:   BigUint,
    pub remaining: BigUint,
    pub f_type: FundType,
}

impl<BigUint:BigUintApi> Encode for SwapCheckpoint<BigUint> {
    fn dep_encode_to<O: Output>(&self, dest: &mut O)  -> Result<(), EncodeError> {
        self.initial.dep_encode_to(dest)?;
        self.remaining.dep_encode_to(dest)?;
        self.f_type.dep_encode_to(dest)?;
        Ok(())
    }
}

impl<BigUint:BigUintApi> Decode for SwapCheckpoint<BigUint> {
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(SwapCheckpoint{
            initial:   BigUint::dep_decode(input)?,
            remaining: BigUint::dep_decode(input)?,
            f_type:    FundType::dep_decode(input)?,
        })
    }
}

#[derive(PartialEq)]
pub struct ExtendedComputation<BigUint:BigUintApi> {
    pub total_delegation_cap: BigUint,
    pub remaining_swap_waiting_to_active: BigUint,
    pub remaining_swap_active_to_def_p: BigUint,
    pub remaining_swap_unstaked_to_def_p: BigUint,
    
    pub computation_type: ComputationType,
    pub step: ComputationStep<BigUint>,
}

impl<BigUint:BigUintApi> ExtendedComputation<BigUint> {
    pub fn is_empty(&self) -> bool {
        self.step == ComputationStep::None &&
        self.remaining_swap_waiting_to_active == 0 &&
        self.remaining_swap_active_to_def_p == 0 &&
        self.remaining_swap_unstaked_to_def_p == 0
    }
}

impl<BigUint:BigUintApi> Encode for ExtendedComputation<BigUint> {
    fn using_top_encoded<F: FnOnce(&[u8])>(&self, f: F) -> Result<(), EncodeError> {
        // delete storage when the balance reaches 0
        // also require links to have been reset (this check is not strictly necessary, but improves safety)
        if self.is_empty() {
            f(&[]);
        } else {
            let mut result: Vec<u8> = Vec::new();
            self.dep_encode_to(&mut result)?;
            f(result.as_slice());
        }
        Ok(())
    }

	fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.total_delegation_cap.dep_encode_to(dest)?;
        self.remaining_swap_waiting_to_active.dep_encode_to(dest)?;
        self.remaining_swap_active_to_def_p.dep_encode_to(dest)?;
        self.remaining_swap_unstaked_to_def_p.dep_encode_to(dest)?;
        self.computation_type.dep_encode_to(dest)?;
        self.step.dep_encode_to(dest)?;
        Ok(())
	}
}

impl<BigUint:BigUintApi> Decode for ExtendedComputation<BigUint> {
    fn top_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        if input.remaining_len() == 0 {
            // does not exist in storage 
            Ok(ExtendedComputation{
                total_delegation_cap: BigUint::zero(),
                remaining_swap_waiting_to_active: BigUint::zero(),
                remaining_swap_active_to_def_p: BigUint::zero(),
                remaining_swap_unstaked_to_def_p: BigUint::zero(),
                computation_type: ComputationType::ModifyTotalDelegationCap,
                step: ComputationStep::None,
            })
        } else {
            let result = Self::dep_decode(input)?;
            if input.remaining_len() > 0 {
                return Err(DecodeError::InputTooLong);
            }
            Ok(result)
        }
    }

    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(ExtendedComputation {
            total_delegation_cap: BigUint::dep_decode(input)?,
            remaining_swap_waiting_to_active: BigUint::dep_decode(input)?,
            remaining_swap_active_to_def_p: BigUint::dep_decode(input)?,
            remaining_swap_unstaked_to_def_p: BigUint::dep_decode(input)?,
            computation_type: ComputationType::dep_decode(input)?,
            step: ComputationStep::dep_decode(input)?,
        })
    }
}


#[derive(PartialEq)]
pub enum ComputationStep<BigUint:BigUintApi> {
    None,
    ComputeAllRewards(ComputeAllRewardsData<BigUint>),
    SwapWaitingToActive,
    SwapUnstakedToDeferredPayment,
    SwapActiveToDeferredPayment,
}

impl<BigUint:BigUintApi> Encode for ComputationStep<BigUint> {
    fn dep_encode_to<O: Output>(&self, dest: &mut O)  -> Result<(), EncodeError> {
        match self {
            ComputationStep::None => { dest.push_byte(0); },
            ComputationStep::ComputeAllRewards(data) => {
                dest.push_byte(1);
                data.dep_encode_to(dest)?;
            },
            ComputationStep::SwapWaitingToActive => { dest.push_byte(2); },
            ComputationStep::SwapUnstakedToDeferredPayment => { dest.push_byte(3); },
            ComputationStep::SwapActiveToDeferredPayment => { dest.push_byte(4); },
        }
        Ok(())
    }
}

impl<BigUint:BigUintApi> Decode for ComputationStep<BigUint> {
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        let discriminant = input.read_byte()?;
        match discriminant {
            0 => Ok(ComputationStep::None),
            1 => Ok(ComputationStep::ComputeAllRewards(ComputeAllRewardsData::dep_decode(input)?)),
            2 => Ok(ComputationStep::SwapWaitingToActive),
            3 => Ok(ComputationStep::SwapUnstakedToDeferredPayment),
            4 => Ok(ComputationStep::SwapActiveToDeferredPayment),
            _ => Err(DecodeError::InvalidValue),
        }
    }
}

#[derive(PartialEq)]
pub enum ComputationType {
    ModifyTotalDelegationCap,
    ChangeServiceFee,
}

impl Encode for ComputationType {
    fn dep_encode_to<O: Output>(&self, dest: &mut O)  -> Result<(), EncodeError> {
        match self {
            ComputationType::ModifyTotalDelegationCap => { dest.push_byte(0); },
            ComputationType::ChangeServiceFee => { dest.push_byte(1); },
        }
        Ok(())
    }
}

impl Decode for ComputationType {
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        let discriminant = input.read_byte()?;
        match discriminant {
            0 => Ok(ComputationType::ModifyTotalDelegationCap),
            1 => Ok(ComputationType::ChangeServiceFee),
            _ => Err(DecodeError::InvalidValue),
        }
    }
}

#[derive(PartialEq)]
pub struct ComputeAllRewardsData<BigUint:BigUintApi> {
    pub last_id:              usize,
    pub sum_unclaimed:        BigUint,
    pub epoch:                u64,
}

impl<BigUint:BigUintApi> ComputeAllRewardsData<BigUint> {
    pub fn new(epoch: u64) -> ComputeAllRewardsData<BigUint> {
        ComputeAllRewardsData {
            last_id: 0,
            sum_unclaimed: BigUint::zero(),
            epoch,
        }
    }
}

impl<BigUint:BigUintApi> Encode for ComputeAllRewardsData<BigUint> {
	fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.last_id.dep_encode_to(dest)?;
        self.sum_unclaimed.dep_encode_to(dest)?;
        self.epoch.dep_encode_to(dest)?;
        Ok(())
	}
}

impl<BigUint:BigUintApi> Decode for ComputeAllRewardsData<BigUint> {
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(ComputeAllRewardsData{
            last_id: usize::dep_decode(input)?,
            sum_unclaimed: BigUint::dep_decode(input)?,
            epoch: u64::dep_decode(input)?,
        })
    }
}


