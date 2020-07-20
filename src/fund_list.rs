use elrond_wasm::elrond_codec::*;
use elrond_wasm::Vec;
use elrond_wasm::BigUintApi;

use super::fund_item::*;

/// A list of fund items, that also works as a queue.
/// When an individual item's balance drops to zero,
/// it is no longer serialized to storage.
/// This is how we manage to delete items and free storage without moving the items that come after.
pub struct FundList<BigUint:BigUintApi> (pub Vec<FundItem<BigUint>>);

impl<BigUint:BigUintApi> FundList<BigUint> {
    /// Will coalesce consecutively pushed items if they are compatible to each other.
    /// Regular Vec push otherwise.
    pub fn push(&mut self, item: FundItem<BigUint>) {
        if !self.0.is_empty() {
            let last_index = self.0.len() - 1;
            let last = &mut self.0[last_index];
            if FundInfo::can_coalesce(&last.info, &item.info) {
                last.balance += item.balance;
                return;
            }
        }
        self.0.push(item);
    }
}

/// Serializes identically to a Vec, entries before start index are ignored.
/// Also entries without a balance are dropped.
/// Does not implement dep_encode_to because unlikve Vec, the length is not known ahead of time,
/// and beccause we never need to embed it into another struct.
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
