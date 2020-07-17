use elrond_wasm::elrond_codec::*;
use elrond_wasm::Vec;
// use elrond_wasm::Queue;
use elrond_wasm::BigUintApi;

// use super::fund_type::*;
use super::fund_item::*;


pub struct FundList<BigUint:BigUintApi> (pub Vec<FundItem<BigUint>>);

impl<BigUint:BigUintApi> FundList<BigUint> {
    #[inline]
    pub fn push(&mut self, item: FundItem<BigUint>) {
        self.0.push(item);
    }
}

/// Serializes identically to a Vec, entries before start index are ignored.
impl<BigUint:BigUintApi> Encode for FundList<BigUint> {
    #[inline]
	fn using_top_encoded<F: FnOnce(&[u8])>(&self, f: F) -> Result<(), EncodeError> {
        let mut result: Vec<u8> = Vec::new();
        for x in self.0.as_slice() {
            if x.balance > 0 {
                x.dep_encode_to(&mut result)?;
            }
        }
        f(result.as_slice());
        Ok(())
    }
    
	fn dep_encode_to<O: Output>(&self, _dest: &mut O) -> Result<(), EncodeError> {
		Result::Err(EncodeError::Static(&b"FundList embedding not allowed"[..]))
	}
}

/// Deserializes like a Vec.
impl<BigUint:BigUintApi> Decode for FundList<BigUint> {
	#[inline]
	fn top_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(FundList(Vec::<FundItem<BigUint>>::top_decode(input)?))
    }
    
    #[inline]
	fn dep_decode<I: Input>(_input: &mut I) -> Result<Self, DecodeError> {
        Result::Err(DecodeError::Static(&b"FundList embedding not allowed"[..]))
    }
}
