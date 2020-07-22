use elrond_wasm::esd_light::*;

/// Contract-wide status of all stake.
#[derive(PartialEq)]
pub enum StakeState {
    /// Users can add or withdraw stake. 
    /// The owner can change the number of nodes and the BLS keys.
    /// All stake resides in the delegation contract.
    /// No rewards arrive from the protocol.
    OpenForStaking,

    /// Stake call to auction sent.
    PendingActivation,

    /// Stake is locked in the protocol and rewards are coming in.
    /// Users cannot withdraw stake, but they can exchange their share of the total stake amongst each other.
    Active,

    /// UnStake call to auction sent.
    PendingDectivation,

    /// Same as Active, but no rewards are coming in.
    /// This is necessary for a period of time before the stake can be retrieved and unlocked.
    UnBondPeriod,

    /// UnBond call to auction sent.
    PendingUnBond,
}

impl StakeState {
    pub fn is_open(&self) -> bool {
        match self {
            StakeState::OpenForStaking => true,
            _ => false,
        }
    }

    fn to_u8(&self) -> u8 {
        match self {
            StakeState::OpenForStaking => 0,
            StakeState::PendingActivation => 1,
            StakeState::Active => 2,
            StakeState::PendingDectivation => 3,
            StakeState::UnBondPeriod => 4,
            StakeState::PendingUnBond => 5,
        }
    }

    fn from_u8(v: u8) -> Result<Self, DecodeError> {
        match v {
            0 => Ok(StakeState::OpenForStaking),
            1 => Ok(StakeState::PendingActivation),
            2 => Ok(StakeState::Active),
            3 => Ok(StakeState::PendingDectivation),
            4 => Ok(StakeState::UnBondPeriod),
            5 => Ok(StakeState::PendingUnBond),
            _ => Err(DecodeError::InvalidValue),
        }
    }
}

impl Encode for StakeState {
    #[inline]
	fn dep_encode_to<O: Output>(&self, dest: &mut O) {
        self.to_u8().dep_encode_to(dest)
	}

	#[inline]
	fn using_top_encoded<F: FnOnce(&[u8])>(&self, f: F) {
        self.to_u8().using_top_encoded(f)
	}
}

impl Decode for StakeState {
	#[inline]
	fn top_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        StakeState::from_u8(u8::top_decode(input)?)
    }
    
    #[inline]
	fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        StakeState::from_u8(u8::dep_decode(input)?)
    }
}
