use elrond_wasm::serde::ser::{Serialize, Serializer};
use elrond_wasm::serde::de::{Deserialize, Deserializer, Visitor, Error};

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
}

impl Serialize for StakeState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let variant_index: u8 = match self {
            StakeState::OpenForStaking => 0,
            StakeState::PendingActivation => 1,
            StakeState::Active => 2,
            StakeState::PendingDectivation => 3,
            StakeState::UnBondPeriod => 4,
            StakeState::PendingUnBond => 5,
        };
        serializer.serialize_u8(variant_index)
    }
}

struct StakeStateVisitor;

impl<'a> Visitor<'a> for StakeStateVisitor {
    type Value = StakeState;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("StakeState")
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match v {
            0 => Ok(StakeState::OpenForStaking),
            1 => Ok(StakeState::PendingActivation),
            2 => Ok(StakeState::Active),
            3 => Ok(StakeState::PendingDectivation),
            4 => Ok(StakeState::UnBondPeriod),
            5 => Ok(StakeState::PendingUnBond),
            _ => Err(E::custom("invalid value")),
        }
    }
}

impl<'de> Deserialize<'de> for StakeState {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<StakeState, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u8(StakeStateVisitor)
    }
}
