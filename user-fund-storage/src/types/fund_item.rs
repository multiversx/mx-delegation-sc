use elrond_wasm::elrond_codec::*;
use elrond_wasm::BigUintApi;
use elrond_wasm::Vec;

use super::fund_type::*;

/// A unit of balance, usually stake.
/// Contains a description of the source/intent of the funds, together with a balance.
#[derive(PartialEq, Debug)]
pub struct FundItem<BigUint:BigUintApi> {
    pub fund_desc: FundDescription,
    pub user_id: usize,
    pub balance: BigUint,
    pub type_list_next: usize,
    pub type_list_prev: usize,
    pub user_list_next: usize,
    pub user_list_prev: usize,
}

#[derive(PartialEq, Debug)]
pub struct FundsListInfo<BigUint:BigUintApi> {
    pub total_balance: BigUint,
    pub first: usize,
    pub last: usize,
}

impl<BigUint:BigUintApi> FundsListInfo<BigUint> {
    pub fn is_empty(&self) -> bool {
        self.first == 0
    }
}

impl<BigUint:BigUintApi> Encode for FundItem<BigUint> {
    fn using_top_encoded<F: FnOnce(&[u8])>(&self, f: F) -> Result<(), EncodeError> {
        if self.balance == 0 {
            // delete storage if the balance reaches 0
            f(&[]);
        } else {
            let mut result: Vec<u8> = Vec::new();
            self.dep_encode_to(&mut result)?;
            f(result.as_slice());
        }
        Ok(())
    }

	fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.fund_desc.dep_encode_to(dest)?;
        self.user_id.dep_encode_to(dest)?;
        self.balance.dep_encode_to(dest)?;
        self.type_list_next.dep_encode_to(dest)?;
        self.type_list_prev.dep_encode_to(dest)?;
        self.user_list_next.dep_encode_to(dest)?;
        self.user_list_prev.dep_encode_to(dest)?;
        Ok(())
	}
}

impl<BigUint:BigUintApi> Decode for FundItem<BigUint> {
    fn top_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        if input.remaining_len() == 0 {
            // does not exist in storage 
            Ok(FundItem{
                fund_desc: FundDescription::WithdrawOnly,
                user_id: 0,
                balance: BigUint::zero(),
                type_list_next: 0,
                type_list_prev: 0,
                user_list_next: 0,
                user_list_prev: 0,
            })
        } else {
            let result = Self::dep_decode(input)?;
            if input.remaining_len() > 0 {
                return Err(DecodeError::InputTooLong);
            }
            Ok(result)
        }
    }

    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(FundItem {
            fund_desc: FundDescription::dep_decode(input)?,
            user_id: usize::dep_decode(input)?,
            balance: BigUint::dep_decode(input)?,
            type_list_next: usize::dep_decode(input)?,
            type_list_prev: usize::dep_decode(input)?,
            user_list_next: usize::dep_decode(input)?,
            user_list_prev: usize::dep_decode(input)?,
        })
    }
}

impl<BigUint:BigUintApi> Encode for FundsListInfo<BigUint> {
    fn using_top_encoded<F: FnOnce(&[u8])>(&self, f: F) -> Result<(), EncodeError> {
        if self.total_balance == 0 {
            // delete storage if the total balance reaches 0
            f(&[]);
        } else {
            let mut result: Vec<u8> = Vec::new();
            self.dep_encode_to(&mut result)?;
            f(result.as_slice());
        }
        Ok(())
    }

	fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.total_balance.dep_encode_to(dest)?;
        self.first.dep_encode_to(dest)?;
        self.last.dep_encode_to(dest)?;
        Ok(())
	}
}

impl<BigUint:BigUintApi> Decode for FundsListInfo<BigUint> {
    fn top_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        if input.remaining_len() == 0 {
            // does not exist in storage 
            Ok(FundsListInfo {
                total_balance: BigUint::zero(),
                first: 0,
                last: 0,
            })
        } else {
            let result = Self::dep_decode(input)?;
            if input.remaining_len() > 0 {
                return Err(DecodeError::InputTooLong);
            }
            Ok(result)
        }
    }

    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(FundsListInfo {
            total_balance: BigUint::dep_decode(input)?,
            first: usize::dep_decode(input)?,
            last: usize::dep_decode(input)?,
        })
    }
}
