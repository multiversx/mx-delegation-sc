use elrond_wasm::elrond_codec::*;

/// Status of a node.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum NodeState {
    /// Node is registered in delegation, but not in the auction SC.
    Inactive,

    /// Stake call to auction sent, but callback not yet received.
    PendingActivation,

    /// Node stake was sent to the auction SC, but the transaction failed for the node.
    /// No longer used.
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
            NodeState::ActivationFailed => 2,
            NodeState::Active => 3,
            NodeState::PendingDeactivation => 4,
            NodeState::UnBondPeriod{..} => 5,
            NodeState::PendingUnBond{..} => 6,
            NodeState::Removed => 7,
        }
    }
}

impl NestedEncode for NodeState {
    fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
        match self {
            NodeState::Inactive => { dest.push_byte(0); },
            NodeState::PendingActivation => { dest.push_byte(1); },
            NodeState::ActivationFailed => { dest.push_byte(2); },
            NodeState::Active => { dest.push_byte(3); },
            NodeState::PendingDeactivation => { dest.push_byte(4); },
            NodeState::UnBondPeriod{ started } => {
                dest.push_byte(5);
                started.dep_encode(dest)?;
            },
            NodeState::PendingUnBond{ unbond_started } => {
                dest.push_byte(6);
                unbond_started.dep_encode(dest)?;
            },
            NodeState::Removed => { dest.push_byte(7); },
        }
        Ok(())
    }

    fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(&self, dest: &mut O, c: ExitCtx, exit: fn(ExitCtx, EncodeError) -> !) {
        match self {
            NodeState::Inactive => { dest.push_byte(0); },
            NodeState::PendingActivation => { dest.push_byte(1); },
            NodeState::ActivationFailed => { dest.push_byte(2); },
            NodeState::Active => { dest.push_byte(3); },
            NodeState::PendingDeactivation => { dest.push_byte(4); },
            NodeState::UnBondPeriod{ started } => {
                dest.push_byte(5);
                started.dep_encode_or_exit(dest, c.clone(), exit);
            },
            NodeState::PendingUnBond{ unbond_started } => {
                dest.push_byte(6);
                unbond_started.dep_encode_or_exit(dest, c.clone(), exit);
            },
            NodeState::Removed => { dest.push_byte(7); },
        }
    }
}

impl TopEncode for NodeState {
    #[inline]
    fn top_encode<O: TopEncodeOutput>(&self, output: O) -> Result<(), EncodeError> {
        top_encode_from_nested(self, output)
    }

    #[inline]
    fn top_encode_or_exit<O: TopEncodeOutput, ExitCtx: Clone>(&self, output: O, c: ExitCtx, exit: fn(ExitCtx, EncodeError) -> !) {
        top_encode_from_nested_or_exit(self, output, c, exit);
    }
}

impl NestedDecode for NodeState {
    fn dep_decode<I: NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
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
            _ => Err(DecodeError::INVALID_VALUE),
        }
    }

    fn dep_decode_or_exit<I: NestedDecodeInput, ExitCtx: Clone>(input: &mut I, c: ExitCtx, exit: fn(ExitCtx, DecodeError) -> !) -> Self {
        let discriminant = input.read_byte_or_exit(c.clone(), exit);
        match discriminant {
            0 => NodeState::Inactive,
            1 => NodeState::PendingActivation,
            2 => NodeState::ActivationFailed,
            3 => NodeState::Active,
            4 => NodeState::PendingDeactivation,
            5 => NodeState::UnBondPeriod{
                started: u64::dep_decode_or_exit(input, c.clone(), exit)
            },
            6 => NodeState::PendingUnBond{
                unbond_started: u64::dep_decode_or_exit(input, c.clone(), exit)
            },
            7 => NodeState::Removed,
            _ => exit(c, DecodeError::INVALID_VALUE),
        }
    }
}

impl TopDecode for NodeState {
    fn top_decode<I: TopDecodeInput>(input: I) -> Result<Self, DecodeError> {
        top_decode_from_nested(input)
    }

    fn top_decode_or_exit<I: TopDecodeInput, ExitCtx: Clone>(input: I, c: ExitCtx, exit: fn(ExitCtx, DecodeError) -> !) -> Self {
        top_decode_from_nested_or_exit(input, c, exit)
    }
}
