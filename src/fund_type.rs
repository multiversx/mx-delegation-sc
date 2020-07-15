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
    ActiveForSale{ bl_nonce: u64 },

    /// UnStake call to auction sent.
    PendingDeactivation{ requested_unstake: bool },

    /// This is necessary for a period of time before the stake can be retrieved and unlocked.
    UnBondPeriod{ requested_unstake: bool },

    /// UnBond call to auction sent.
    PendingUnBond{ requested_unstake: bool },

    /// Node stake was sent to the auction SC, but the transaction failed for the node.
    ActivationFailed,

    DeferredPayment{ created: u64 },

    Reward,
}

impl FundType {
    pub fn discriminant(&self) -> u8 {
        match self {
            FundType::Free{..} => 0,
            FundType::PendingActivation => 1,
            FundType::Active => 2,
            FundType::ActiveForSale{..} => 3,
            FundType::PendingDeactivation{..} => 3,
            FundType::UnBondPeriod{..} => 4,
            FundType::PendingUnBond{..} => 5,
            FundType::ActivationFailed => 7,
            FundType::DeferredPayment{..} => 8,
            FundType::Reward{..} => 9,
        }
    }
}

impl Encode for FundType {
    #[inline]
	fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        match self {
            FundType::Free{..} => { dest.push_byte(0u8); },
            FundType::PendingActivation => { dest.push_byte(1u8); },
            FundType::Active => { dest.push_byte(2u8); },
            FundType::ActiveForSale{..} => { dest.push_byte(3u8); },
            FundType::PendingDeactivation{..} => { dest.push_byte(3u8); },
            FundType::UnBondPeriod{..} => { dest.push_byte(4u8); },
            FundType::PendingUnBond{..} => { dest.push_byte(5u8); },
            FundType::ActivationFailed => { dest.push_byte(7u8); },
            FundType::DeferredPayment{..} => { dest.push_byte(8u8); },
            FundType::Reward => { dest.push_byte(9u8); },
        }
        Ok(())
	}
}

impl Decode for FundType {
    #[inline]
	fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        let discriminant = input.read_byte()?;
        match discriminant {
            0 => Ok(FundType::Free{ requested_unstake: false }),
            1 => Ok(FundType::PendingActivation),
            2 => Ok(FundType::Active),
            3 => Ok(FundType::ActiveForSale{ bl_nonce: 0 }),
            4 => Ok(FundType::PendingDeactivation{ requested_unstake: false }),
            5 => Ok(FundType::UnBondPeriod{ requested_unstake: false }),
            6 => Ok(FundType::PendingUnBond{ requested_unstake: false }),
            7 => Ok(FundType::ActivationFailed),
            8 => Ok(FundType::DeferredPayment{ created: 0u64 }),
            9 => Ok(FundType::Reward),
            _ => Err(DecodeError::InvalidValue),
        }
    }
}


