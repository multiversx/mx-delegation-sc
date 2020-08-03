use elrond_wasm::elrond_codec::*;
use elrond_wasm::BigUintApi;

use super::fund_type::*;

/// Describes a unit of balance, usually stake.
/// Contains the descriptive part of a fund bucket, without the balance part.
pub struct FundInfo {
    pub user_id: usize,
    pub fund_type: FundType,
}

impl FundInfo {
    /// Pushing identical items at the end of a list, brings them together in 1 item.
    /// This is to save storage space.
    pub fn can_coalesce(f1: &FundInfo, f2: &FundInfo) -> bool {
        f1.user_id == f2.user_id && f1.fund_type == f2.fund_type
    }
}

/// A unit of balance, usually stake.
/// Contains a description of the source/intent of the funds, together with a balance.
pub struct FundItem<BigUint:BigUintApi> {
    pub info: FundInfo,
    pub balance: BigUint,
}

impl<BigUint:BigUintApi> Encode for FundItem<BigUint> {
	#[inline]
	fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.info.user_id.dep_encode_to(dest)?;
        self.info.fund_type.dep_encode_to(dest)?;
        self.balance.dep_encode_to(dest)?;
        Ok(())
	}
}

impl<BigUint:BigUintApi> Decode for FundItem<BigUint> {
    #[inline]
	fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(FundItem {
            info: FundInfo {
                user_id: usize::dep_decode(input)?,
                fund_type: FundType::dep_decode(input)?,
            },
            balance: BigUint::dep_decode(input)?,
        })
    }
}
