use elrond_wasm::esd_light::*;

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
        }
    }

    fn from_u8(v: u8) -> Result<Self, DeError> {
        match v {
            0 => Ok(UserStakeState::Inactive),
            1 => Ok(UserStakeState::PendingActivation),
            2 => Ok(UserStakeState::Active),
            3 => Ok(UserStakeState::PendingDeactivation),
            4 => Ok(UserStakeState::UnBondPeriod),
            5 => Ok(UserStakeState::PendingUnBond),
            6 => Ok(UserStakeState::WithdrawOnly),
            _ => Err(DeError::InvalidValue),
        }
    }
}

impl Encode for UserStakeState {
    #[inline]
	fn dep_encode_to<O: Output>(&self, dest: &mut O) {
        self.to_u8().dep_encode_to(dest)
	}

	#[inline]
	fn using_top_encoded<F: FnOnce(&[u8])>(&self, f: F) {
        self.to_u8().using_top_encoded(f)
	}
}

impl Decode for UserStakeState {
	#[inline]
	fn top_decode<I: Input>(input: &mut I) -> Result<Self, DeError> {
        UserStakeState::from_u8(u8::top_decode(input)?)
    }
    
    #[inline]
	fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DeError> {
        UserStakeState::from_u8(u8::dep_decode(input)?)
    }
}