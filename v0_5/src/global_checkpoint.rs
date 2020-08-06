use elrond_wasm::elrond_codec::*;
use elrond_wasm::BigUintApi;

#[derive(Clone)]
#[derive(PartialEq)]
pub struct GlobalCheckpoint<BigUint:BigUintApi> {
    pub total_delegation_cap: BigUint,
    pub finished:             bool,
    pub last_id:              usize,
    pub sum_unclaimed:        BigUint,
}

impl<BigUint:BigUintApi> Encode for GlobalCheckpoint<BigUint> {
    #[inline]
    fn dep_encode_to<O: Output>(&self, dest: &mut O)  -> Result<(), EncodeError> {
        self.total_delegation_cap.dep_encode_to(dest)?;
        self.finished.dep_encode_to(dest)?;
        self.last_id.dep_encode_to(dest)?;
        self.sum_unclaimed.dep_encode_to(dest)?;
        Ok(())
    }
}

impl<BigUint:BigUintApi> Decode for GlobalCheckpoint<BigUint> {
    #[inline]
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(GlobalCheckpoint{
            total_delegation_cap: BigUint::dep_decode(input)?,
            finished:             bool::dep_decode(input)?,
            last_id:              usize::dep_decode(input)?,
            sum_unclaimed:        BigUint::dep_decode(input)?,
        })
    }
}
