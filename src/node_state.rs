use elrond_wasm::esd_light::*;

/// Status of a node.
#[derive(PartialEq, Clone, Copy)]
pub enum NodeState {
    /// Node is registered in delegation, but not in the auction SC.
    Inactive,

    /// Stake call to auction sent, but callback not yet received.
    PendingActivation,

    /// Node is registered in the auction SC, active and producing rewards.
    Active,

    /// UnStake call to auction sent, but callback not yet received.
    PendingDeactivation,

    /// Same as Active, but no rewards are coming in.
    /// This is necessary for a period of time before the stake can be retrieved and unlocked.
    UnBondPeriod,

    /// UnBond call to auction sent, but callback not yet received.
    PendingUnBond,

    /// Node completely removed from the delegation contract.
    /// TODO: properly remove nodes instead of just flagging them.
    Removed,

    /// Node stake was sent to the auction SC, but the transaction failed for the node.
    ActivationFailed,
}

impl NodeState {
    pub fn to_u8(&self) -> u8 {
        match self {
            NodeState::Inactive => 0,
            NodeState::PendingActivation => 1,
            NodeState::Active => 2,
            NodeState::PendingDeactivation => 3,
            NodeState::UnBondPeriod => 4,
            NodeState::PendingUnBond => 5,
            NodeState::Removed => 6,
            NodeState::ActivationFailed => 7,
        }
    }

    fn from_u8(v: u8) -> Result<Self, DecodeError> {
        match v {
            0 => Ok(NodeState::Inactive),
            1 => Ok(NodeState::PendingActivation),
            2 => Ok(NodeState::Active),
            3 => Ok(NodeState::PendingDeactivation),
            4 => Ok(NodeState::UnBondPeriod),
            5 => Ok(NodeState::PendingUnBond),
            6 => Ok(NodeState::Removed),
            7 => Ok(NodeState::ActivationFailed),
            _ => Err(DecodeError::InvalidValue),
        }
    }
}

impl Encode for NodeState {
    #[inline]
	fn dep_encode_to<O: Output>(&self, dest: &mut O) {
        self.to_u8().dep_encode_to(dest)
	}

	#[inline]
	fn using_top_encoded<F: FnOnce(&[u8])>(&self, f: F) {
        self.to_u8().using_top_encoded(f)
	}
}

impl Decode for NodeState {
	#[inline]
	fn top_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        NodeState::from_u8(u8::top_decode(input)?)
    }
    
    #[inline]
	fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        NodeState::from_u8(u8::dep_decode(input)?)
    }
}
