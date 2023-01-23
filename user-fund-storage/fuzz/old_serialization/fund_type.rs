use multiversx_sc::codec::*;

multiversx_sc::derive_imports!();

#[derive(TypeAbi, PartialEq, Clone, Copy, Debug)]
pub enum FundDescription {
    /// Funds that can only be extracted from contract. Will never be used as stake.
    WithdrawOnly,

    /// Inactive stake, waiting in the queue to be activated.
    Waiting {
        created: u64,
    },

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
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FundType {
    /// Funds that can only be extracted from contract. Will never be used as stake.
    WithdrawOnly,

    /// Inactive stake, waiting in the queue to be activated.
    Waiting,

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
        match self {
            FundType::WithdrawOnly | FundType::DeferredPayment => true,
            _ => false,
        }
    }

    pub fn is_stake(&self) -> bool {
        match self {
            FundType::Waiting | FundType::Active | FundType::UnStaked => true,
            _ => false,
        }
    }

    pub fn funds_in_contract(&self) -> bool {
        match self {
            FundType::WithdrawOnly | FundType::Waiting | FundType::DeferredPayment => true,
            _ => false,
        }
    }
}

pub const DISCR_WITHDRAW_ONLY: u8 = 0;
pub const DISCR_WAITING: u8 = 1;
pub const DISCR_PENDING_ACT: u8 = 2;
pub const DISCR_ACTIVE_FAILED: u8 = 3;
pub const DISCR_ACTIVE: u8 = 4;
pub const DISCR_UNSTAKED: u8 = 5;
pub const DISCR_DEF_PAYMENT: u8 = 6;
pub const NR_DISCRIMINANTS: u8 = 7;

impl FundDescription {
    #[inline]
    pub fn discriminant(&self) -> u8 {
        self.fund_type().discriminant()
    }

    pub fn fund_type(&self) -> FundType {
        match self {
            FundDescription::WithdrawOnly => FundType::WithdrawOnly,
            FundDescription::Waiting { .. } => FundType::Waiting,
            FundDescription::Active => FundType::Active,
            FundDescription::UnStaked { .. } => FundType::UnStaked,
            FundDescription::DeferredPayment { .. } => FundType::DeferredPayment,
        }
    }
}

impl FundType {
    pub fn discriminant(&self) -> u8 {
        match self {
            FundType::WithdrawOnly => DISCR_WITHDRAW_ONLY,
            FundType::Waiting => DISCR_WAITING,
            FundType::Active => DISCR_ACTIVE,
            FundType::UnStaked => DISCR_UNSTAKED,
            FundType::DeferredPayment => DISCR_DEF_PAYMENT,
        }
    }
}

impl NestedEncode for FundDescription {
    fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
        match self {
            FundDescription::WithdrawOnly => {
                dest.push_byte(DISCR_WITHDRAW_ONLY);
            }
            FundDescription::Waiting { created } => {
                dest.push_byte(DISCR_WAITING);
                created.dep_encode(dest)?;
            }
            FundDescription::Active => {
                dest.push_byte(DISCR_ACTIVE);
            }
            FundDescription::UnStaked { created } => {
                dest.push_byte(DISCR_UNSTAKED);
                created.dep_encode(dest)?;
            }
            FundDescription::DeferredPayment { created } => {
                dest.push_byte(DISCR_DEF_PAYMENT);
                created.dep_encode(dest)?;
            }
        }
        Ok(())
    }

    #[allow(clippy::redundant_clone)]
    fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
        &self,
        dest: &mut O,
        c: ExitCtx,
        exit: fn(ExitCtx, EncodeError) -> !,
    ) {
        match self {
            FundDescription::WithdrawOnly => {
                dest.push_byte(DISCR_WITHDRAW_ONLY);
            }
            FundDescription::Waiting { created } => {
                dest.push_byte(DISCR_WAITING);
                created.dep_encode_or_exit(dest, c.clone(), exit);
            }
            FundDescription::Active => {
                dest.push_byte(DISCR_ACTIVE);
            }
            FundDescription::UnStaked { created } => {
                dest.push_byte(DISCR_UNSTAKED);
                created.dep_encode_or_exit(dest, c.clone(), exit);
            }
            FundDescription::DeferredPayment { created } => {
                dest.push_byte(DISCR_DEF_PAYMENT);
                created.dep_encode_or_exit(dest, c.clone(), exit);
            }
        }
    }
}

impl TopEncode for FundDescription {
    #[inline]
    fn top_encode<O: TopEncodeOutput>(&self, output: O) -> Result<(), EncodeError> {
        top_encode_from_nested(self, output)
    }

    #[inline]
    fn top_encode_or_exit<O: TopEncodeOutput, ExitCtx: Clone>(
        &self,
        output: O,
        c: ExitCtx,
        exit: fn(ExitCtx, EncodeError) -> !,
    ) {
        top_encode_from_nested_or_exit(self, output, c, exit);
    }
}

impl NestedDecode for FundDescription {
    fn dep_decode<I: NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
        let discriminant = input.read_byte()?;
        match discriminant {
            DISCR_WITHDRAW_ONLY => Ok(FundDescription::WithdrawOnly),
            DISCR_WAITING => Ok(FundDescription::Waiting {
                created: u64::dep_decode(input)?,
            }),
            DISCR_ACTIVE => Ok(FundDescription::Active),
            DISCR_UNSTAKED => Ok(FundDescription::UnStaked {
                created: u64::dep_decode(input)?,
            }),
            DISCR_DEF_PAYMENT => Ok(FundDescription::DeferredPayment {
                created: u64::dep_decode(input)?,
            }),
            _ => Err(DecodeError::INVALID_VALUE),
        }
    }

    #[allow(clippy::redundant_clone)]
    fn dep_decode_or_exit<I: NestedDecodeInput, ExitCtx: Clone>(
        input: &mut I,
        c: ExitCtx,
        exit: fn(ExitCtx, DecodeError) -> !,
    ) -> Self {
        let discriminant = input.read_byte_or_exit(c.clone(), exit);
        match discriminant {
            DISCR_WITHDRAW_ONLY => FundDescription::WithdrawOnly,
            DISCR_WAITING => FundDescription::Waiting {
                created: u64::dep_decode_or_exit(input, c.clone(), exit),
            },
            DISCR_ACTIVE => FundDescription::Active,
            DISCR_UNSTAKED => FundDescription::UnStaked {
                created: u64::dep_decode_or_exit(input, c.clone(), exit),
            },
            DISCR_DEF_PAYMENT => FundDescription::DeferredPayment {
                created: u64::dep_decode_or_exit(input, c.clone(), exit),
            },
            _ => exit(c, DecodeError::INVALID_VALUE),
        }
    }
}

impl TopDecode for FundDescription {
    fn top_decode<I: TopDecodeInput>(input: I) -> Result<Self, DecodeError> {
        top_decode_from_nested(input)
    }

    fn top_decode_or_exit<I: TopDecodeInput, ExitCtx: Clone>(
        input: I,
        c: ExitCtx,
        exit: fn(ExitCtx, DecodeError) -> !,
    ) -> Self {
        top_decode_from_nested_or_exit(input, c, exit)
    }
}

impl NestedEncode for FundType {
    fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
        match self {
            FundType::WithdrawOnly => {
                dest.push_byte(DISCR_WITHDRAW_ONLY);
            }
            FundType::Waiting => {
                dest.push_byte(DISCR_WAITING);
            }
            FundType::Active => {
                dest.push_byte(DISCR_ACTIVE);
            }
            FundType::UnStaked => {
                dest.push_byte(DISCR_UNSTAKED);
            }
            FundType::DeferredPayment => {
                dest.push_byte(DISCR_DEF_PAYMENT);
            }
        }
        Ok(())
    }

    fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
        &self,
        dest: &mut O,
        _: ExitCtx,
        _: fn(ExitCtx, EncodeError) -> !,
    ) {
        match self {
            FundType::WithdrawOnly => {
                dest.push_byte(DISCR_WITHDRAW_ONLY);
            }
            FundType::Waiting => {
                dest.push_byte(DISCR_WAITING);
            }
            FundType::Active => {
                dest.push_byte(DISCR_ACTIVE);
            }
            FundType::UnStaked => {
                dest.push_byte(DISCR_UNSTAKED);
            }
            FundType::DeferredPayment => {
                dest.push_byte(DISCR_DEF_PAYMENT);
            }
        }
    }
}

impl TopEncode for FundType {
    #[inline]
    fn top_encode<O: TopEncodeOutput>(&self, output: O) -> Result<(), EncodeError> {
        top_encode_from_nested(self, output)
    }

    #[inline]
    fn top_encode_or_exit<O: TopEncodeOutput, ExitCtx: Clone>(
        &self,
        output: O,
        c: ExitCtx,
        exit: fn(ExitCtx, EncodeError) -> !,
    ) {
        top_encode_from_nested_or_exit(self, output, c, exit);
    }
}

impl NestedDecode for FundType {
    fn dep_decode<I: NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
        let discriminant = input.read_byte()?;
        match discriminant {
            DISCR_WITHDRAW_ONLY => Ok(FundType::WithdrawOnly),
            DISCR_WAITING => Ok(FundType::Waiting),
            DISCR_ACTIVE => Ok(FundType::Active),
            DISCR_UNSTAKED => Ok(FundType::UnStaked),
            DISCR_DEF_PAYMENT => Ok(FundType::DeferredPayment),
            _ => Err(DecodeError::INVALID_VALUE),
        }
    }

    fn dep_decode_or_exit<I: NestedDecodeInput, ExitCtx: Clone>(
        input: &mut I,
        c: ExitCtx,
        exit: fn(ExitCtx, DecodeError) -> !,
    ) -> Self {
        let discriminant = input.read_byte_or_exit(c.clone(), exit);
        match discriminant {
            DISCR_WITHDRAW_ONLY => FundType::WithdrawOnly,
            DISCR_WAITING => FundType::Waiting,
            DISCR_ACTIVE => FundType::Active,
            DISCR_UNSTAKED => FundType::UnStaked,
            DISCR_DEF_PAYMENT => FundType::DeferredPayment,
            _ => exit(c, DecodeError::INVALID_VALUE),
        }
    }
}

impl TopDecode for FundType {
    fn top_decode<I: TopDecodeInput>(input: I) -> Result<Self, DecodeError> {
        top_decode_from_nested(input)
    }

    fn top_decode_or_exit<I: TopDecodeInput, ExitCtx: Clone>(
        input: I,
        c: ExitCtx,
        exit: fn(ExitCtx, DecodeError) -> !,
    ) -> Self {
        top_decode_from_nested_or_exit(input, c, exit)
    }
}
