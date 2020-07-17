use elrond_wasm::elrond_codec::*;

// pub enum UnstakeInfo {
//     None,

//     AnnouncedUnstake{ bl_nonce: u64 },

//     UnBondPeriod{ bl_nonce: u64 },
// }

#[derive(PartialEq, Clone, Copy)]
pub enum FundType {
    /// Funds not staked, free to extract from contract.
    Free{ requested_unstake: bool },

    /// Stake sent to auction SC.
    PendingActivation,

    /// Stake is locked in the protocol and rewards are coming in.
    /// Users cannot withdraw stake, but they can exchange their share of the total stake amongst each other.
    Active,

    /// Same as Active, but no rewards are coming in.
    ActiveForSale{ created: u64 },

    /// UnStake call to auction sent.
    PendingDeactivation{ requested_unstake: bool },

    /// This is necessary for a period of time before the stake can be retrieved and unlocked.
    UnBondPeriod{ created: u64, requested_unstake: bool },

    /// UnBond call to auction sent.
    PendingUnBond{ unbond_created: u64, requested_unstake: bool },

    /// Node stake was sent to the auction SC, but the transaction failed for the node.
    ActivationFailed,

    DeferredPayment{ created: u64 },

    // Reward,
}

pub const DISCR_FREE: u8 = 0;
pub const DISCR_PENDING_ACT: u8 = 1;
pub const DISCR_ACTIVE_FAILED: u8 = 2;
pub const DISCR_ACTIVE: u8 = 3;
pub const DISCR_ACTIVE_FOR_SALE: u8 = 4;
pub const DISCR_PENDING_DEACT: u8 = 5;
pub const DISCR_UNBOND: u8 = 6;
pub const DISCR_PENDING_UNBOND: u8 = 7;
pub const DISCR_DEF_PAYMENT: u8 = 8;
pub const DISCR_REWARD: u8 = 9;

impl FundType {
    pub fn discriminant(&self) -> u8 {
        match self {
            FundType::Free{..} => DISCR_FREE,
            FundType::PendingActivation => DISCR_PENDING_ACT,
            FundType::ActivationFailed => DISCR_ACTIVE_FAILED,
            FundType::Active => DISCR_ACTIVE,
            FundType::ActiveForSale{..} => DISCR_ACTIVE_FOR_SALE,
            FundType::PendingDeactivation{..} => DISCR_PENDING_DEACT,
            FundType::UnBondPeriod{..} => DISCR_UNBOND,
            FundType::PendingUnBond{..} => DISCR_PENDING_UNBOND,
            FundType::DeferredPayment{..} => DISCR_DEF_PAYMENT,
            // FundType::Reward{..} => DISCR_REWARD,
        }
    }

    pub fn is_stake(&self) -> bool {
        match self {
            FundType::Free{..} |
            FundType::PendingActivation |
            FundType::ActivationFailed |
            FundType::Active |
            FundType::ActiveForSale{..} |
            FundType::PendingDeactivation{..} |
            FundType::UnBondPeriod{..} |
            FundType::PendingUnBond{..} => true,
            _ => false,
        }
    }

    pub fn is_active_stake(&self) -> bool {
        match self {
            FundType::Active |
            FundType::ActiveForSale{..} => true,
            _ => false,
        }
    }

    pub fn funds_in_contract(&self) -> bool {
        match self {
            FundType::Free{..} |
            FundType::DeferredPayment { .. } => true,
            _ => false,
        }
    }
}

impl Encode for FundType {
    #[inline]
	fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        match self {
            FundType::Free{ requested_unstake } => {
                dest.push_byte(0u8);
                requested_unstake.dep_encode_to(dest)?;
            },
            FundType::PendingActivation => { dest.push_byte(1u8); },
            FundType::ActivationFailed => { dest.push_byte(2u8); },
            FundType::Active => { dest.push_byte(3u8); },
            FundType::ActiveForSale{ created } => {
                dest.push_byte(4u8);
                created.dep_encode_to(dest)?;
            },
            FundType::PendingDeactivation{ requested_unstake } => {
                dest.push_byte(5u8);
                requested_unstake.dep_encode_to(dest)?;
            },
            FundType::UnBondPeriod{ created, requested_unstake } => {
                dest.push_byte(6u8);
                created.dep_encode_to(dest)?;
                requested_unstake.dep_encode_to(dest)?;
            },
            FundType::PendingUnBond{ unbond_created, requested_unstake } => {
                dest.push_byte(7u8);
                unbond_created.dep_encode_to(dest)?;
                requested_unstake.dep_encode_to(dest)?;
            },
            FundType::DeferredPayment{ created } => {
                dest.push_byte(8u8);
                created.dep_encode_to(dest)?;
            },
            // FundType::Reward => { dest.push_byte(9u8); },
        }
        Ok(())
	}
}

impl Decode for FundType {
    #[inline]
	fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        let discriminant = input.read_byte()?;
        match discriminant {
            0 => Ok(FundType::Free{
                requested_unstake: bool::dep_decode(input)?
            }),
            1 => Ok(FundType::PendingActivation),
            2 => Ok(FundType::ActivationFailed),
            3 => Ok(FundType::Active),
            4 => Ok(FundType::ActiveForSale{
                created: u64::dep_decode(input)?
            }),
            5 => Ok(FundType::PendingDeactivation{
                requested_unstake: bool::dep_decode(input)? 
            }),
            6 => Ok(FundType::UnBondPeriod{
                created: u64::dep_decode(input)?,
                requested_unstake: bool::dep_decode(input)?,
            }),
            7 => Ok(FundType::PendingUnBond{
                unbond_created: u64::dep_decode(input)?,
                requested_unstake: bool::dep_decode(input)?
            }),
            8 => Ok(FundType::DeferredPayment{
                created: u64::dep_decode(input)?
            }),
            // 9 => Ok(FundType::Reward),
            _ => Err(DecodeError::InvalidValue),
        }
    }
}


