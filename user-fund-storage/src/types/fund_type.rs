multiversx_sc::derive_imports!();

#[derive(
    TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Clone, Copy, Debug,
)]
pub enum FundDescription {
    /// Funds that can only be extracted from contract. Will never be used as stake.
    WithdrawOnly,

    /// Inactive stake, waiting in the queue to be activated.
    Waiting {
        created: u64,
    },

    // Unused. Kept for serialization.
    _PendingAct,

    // Unused. Kept for serialization.
    _ActiveFailed,

    /// Stake is locked in the protocol and rewards are coming in.
    /// Users cannot withdraw stake, but they can exchange their share of the total stake amongst each other.
    Active,

    /// Same as Active, but no rewards are coming in.
    UnStaked {
        created: u64,
    },

    DeferredPayment {
        created: u64,
    },
}

/// Same as fund description, but only the enum with no additional data.
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Clone, Copy, Debug)]
pub enum FundType {
    /// Funds that can only be extracted from contract. Will never be used as stake.
    WithdrawOnly,

    /// Inactive stake, waiting in the queue to be activated.
    Waiting,

    // Unused. Kept for serialization.
    _PendingAct,

    // Unused. Kept for serialization.
    _ActiveFailed,

    /// Stake is locked in the protocol and rewards are coming in.
    /// Users cannot withdraw stake, but they can exchange their share of the total stake amongst each other.
    Active,

    /// Same as Active, but no rewards are coming in.
    UnStaked,

    DeferredPayment,
}

impl FundType {
    pub const ALL_TYPES: &'static [FundType] = &[
        FundType::WithdrawOnly,
        FundType::Waiting,
        FundType::Active,
        FundType::UnStaked,
        FundType::DeferredPayment,
    ];

    pub fn allow_coalesce(&self) -> bool {
        matches!(self, FundType::WithdrawOnly | FundType::DeferredPayment)
    }

    pub fn is_stake(&self) -> bool {
        matches!(
            self,
            FundType::Waiting | FundType::Active | FundType::UnStaked
        )
    }

    pub fn funds_in_contract(&self) -> bool {
        matches!(
            self,
            FundType::WithdrawOnly | FundType::Waiting | FundType::DeferredPayment
        )
    }
}

impl FundDescription {
    pub fn fund_type(&self) -> FundType {
        match self {
            FundDescription::WithdrawOnly => FundType::WithdrawOnly,
            FundDescription::Waiting { .. } => FundType::Waiting,
            FundDescription::_PendingAct => FundType::_PendingAct,
            FundDescription::_ActiveFailed => FundType::_ActiveFailed,
            FundDescription::Active => FundType::Active,
            FundDescription::UnStaked { .. } => FundType::UnStaked,
            FundDescription::DeferredPayment { .. } => FundType::DeferredPayment,
        }
    }
}
