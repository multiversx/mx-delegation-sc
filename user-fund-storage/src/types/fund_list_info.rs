use elrond_wasm::elrond_codec::*;
use elrond_wasm::BigUintApi;
use elrond_wasm::Vec;

#[derive(PartialEq, Debug)]
pub struct FundsListInfo<BigUint:BigUintApi> {
    pub total_balance: BigUint,
    pub first: usize,
    pub last: usize,
}

impl<BigUint:BigUintApi> FundsListInfo<BigUint> {
    pub fn is_empty(&self) -> bool {
        self.total_balance == 0 &&
        self.first == 0 &&
        self.last == 0
    }
}

impl<BigUint:BigUintApi> Encode for FundsListInfo<BigUint> {
    fn using_top_encoded<F: FnOnce(&[u8])>(&self, f: F) -> Result<(), EncodeError> {
        if self.is_empty() {
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
