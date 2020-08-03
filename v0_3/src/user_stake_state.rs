use elrond_wasm::elrond_codec::*;

/// Contract-wide status of all stake.
/// Similar to NodeState, but labels the user stake, not the node status.
#[derive(PartialEq, Clone, Copy)]
pub enum UserStakeState {
    /// Node is registered in delegation, but not in the auction SC.
    Inactive,

    /// Stake sent to auction SC.
    PendingActivation,

    /// Stake is locked in the protocol and rewards are coming in.
    /// Users cannot withdraw stake, but they can exchange their share of the total stake amongst each other.
    Active,

    /// UnStake call to auction sent.
    PendingDeactivation,

    /// Same as Active, but no rewards are coming in.
    /// This is necessary for a period of time before the stake can be retrieved and unlocked.
    UnBondPeriod,

    /// UnBond call to auction sent.
    PendingUnBond,

    /// Stake inactive, and also cannot be activated. Can only be withdrawn by delegator.
    WithdrawOnly,

    /// Node stake was sent to the auction SC, but the transaction failed for the node.
    ActivationFailed,

    /// Same as Active, but the owner is offering it for sale.
    /// During this time the stake does not produce rewards.
    /// Instead, the rewards go to the contract owner.
    ActiveForSale,

    /// Same as PendingDeactivation, but originating from stake that is ActiveForSale.
    /// The distinction is necessary in order to be able to correctly revert in case of failure.
    PendingDeactivationFromSale,
}

impl UserStakeState {
    fn to_u8(&self) -> u8 {
        match self {
            UserStakeState::Inactive => 0,
            UserStakeState::PendingActivation => 1,
            UserStakeState::Active => 2,
            UserStakeState::PendingDeactivation => 3,
            UserStakeState::UnBondPeriod => 4,
            UserStakeState::PendingUnBond => 5,
            UserStakeState::WithdrawOnly => 6,
            UserStakeState::ActivationFailed => 7,
            UserStakeState::ActiveForSale => 8,
            UserStakeState::PendingDeactivationFromSale => 9,
        }
    }

    fn from_u8(v: u8) -> Result<Self, DecodeError> {
        match v {
            0 => Ok(UserStakeState::Inactive),
            1 => Ok(UserStakeState::PendingActivation),
            2 => Ok(UserStakeState::Active),
            3 => Ok(UserStakeState::PendingDeactivation),
            4 => Ok(UserStakeState::UnBondPeriod),
            5 => Ok(UserStakeState::PendingUnBond),
            6 => Ok(UserStakeState::WithdrawOnly),
            7 => Ok(UserStakeState::ActivationFailed),
            8 => Ok(UserStakeState::ActiveForSale),
            9 => Ok(UserStakeState::PendingDeactivationFromSale),
            _ => Err(DecodeError::InvalidValue),
        }
    }
}

impl Encode for UserStakeState {
    #[inline]
	fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.to_u8().dep_encode_to(dest)
	}

	#[inline]
	fn using_top_encoded<F: FnOnce(&[u8])>(&self, f: F) -> Result<(), EncodeError> {
        self.to_u8().using_top_encoded(f)
	}
}

impl Decode for UserStakeState {
	#[inline]
	fn top_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        UserStakeState::from_u8(u8::top_decode(input)?)
    }
    
    #[inline]
	fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        UserStakeState::from_u8(u8::dep_decode(input)?)
    }
}
