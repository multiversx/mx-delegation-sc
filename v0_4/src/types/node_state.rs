use elrond_wasm::elrond_codec::*;

/// Status of a node.
#[derive(PartialEq, Clone, Copy)]
pub enum NodeState {
    /// Node is registered in delegation, but not in the auction SC.
    Inactive,

    /// Stake call to auction sent, but callback not yet received.
    PendingActivation,

    /// Node stake was sent to the auction SC, but the transaction failed for the node.
    ActivationFailed,

    /// Node is registered in the auction SC, active and producing rewards.
    Active,

    /// UnStake call to auction sent, but callback not yet received.
    PendingDeactivation,

    /// Same as Active, but no rewards are coming in.
    /// This is necessary for a period of time before the stake can be retrieved and unlocked.
    UnBondPeriod{ started: u64 },

    /// UnBond call to auction sent, but callback not yet received.
    /// `unbond_started` field is needed in case unbonding fails and the UnBondPeriod state needs to be restored.
    PendingUnBond{ unbond_started: u64 },

    /// Node completely removed from the delegation contract.
    Removed,

}

impl NodeState {
    pub fn to_u8(&self) -> u8 {
        match self {
            NodeState::Inactive => 0,
            NodeState::PendingActivation => 1,
            NodeState::Active => 2,
            NodeState::PendingDeactivation => 3,
            NodeState::UnBondPeriod{..} => 4,
            NodeState::PendingUnBond{..} => 5,
            NodeState::Removed => 6,
            NodeState::ActivationFailed => 7,
        }
    }
}

impl Encode for NodeState {
    fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        match self {
            NodeState::Inactive => { dest.push_byte(0); },
            NodeState::PendingActivation => { dest.push_byte(1); },
            NodeState::ActivationFailed => { dest.push_byte(2); },
            NodeState::Active => { dest.push_byte(3); },
            NodeState::PendingDeactivation => { dest.push_byte(4); },
            NodeState::UnBondPeriod{ started } => {
                dest.push_byte(5);
                started.dep_encode_to(dest)?;
            },
            NodeState::PendingUnBond{ unbond_started } => {
                dest.push_byte(6);
                unbond_started.dep_encode_to(dest)?;
            },
            NodeState::Removed => { dest.push_byte(7); },
        }
        Ok(())
	}
}

impl Decode for NodeState {
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        let discriminant = input.read_byte()?;
        match discriminant {
            0 => Ok(NodeState::Inactive),
            1 => Ok(NodeState::PendingActivation),
            2 => Ok(NodeState::ActivationFailed),
            3 => Ok(NodeState::Active),
            4 => Ok(NodeState::PendingDeactivation),
            5 => Ok(NodeState::UnBondPeriod{
                started: u64::dep_decode(input)?
            }),
            6 => Ok(NodeState::PendingUnBond{
                unbond_started: u64::dep_decode(input)?
            }),
            7 => Ok(NodeState::Removed),
            _ => Err(DecodeError::InvalidValue),
        }
    }
}
