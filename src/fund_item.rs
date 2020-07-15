use elrond_wasm::elrond_codec::*;
use elrond_wasm::BigUintApi;

use super::fund_type::*;

pub struct FundInfo {
    pub user_id: usize,
    pub fund_type: FundType,
}

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
