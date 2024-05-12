multiversx_sc::derive_imports!();

/// Status of a node.
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Clone, Copy, Debug)]
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
    UnBondPeriod { started: u64 },

    /// UnBond call to auction sent, but callback not yet received.
    /// `unbond_started` field is needed in case unbonding fails and the UnBondPeriod state needs to be restored.
    PendingUnBond { unbond_started: u64 },

    /// Node completely removed from the delegation contract.
    Removed,
}

impl NodeState {
    pub fn discriminant(&self) -> u8 {
        match self {
            NodeState::Inactive => 0,
            NodeState::PendingActivation => 1,
            NodeState::ActivationFailed => 2,
            NodeState::Active => 3,
            NodeState::PendingDeactivation => 4,
            NodeState::UnBondPeriod { .. } => 5,
            NodeState::PendingUnBond { .. } => 6,
            NodeState::Removed => 7,
        }
    }
}
