use elrond_wasm::elrond_codec::*;

#[derive(Clone)]
#[derive(PartialEq)]
pub struct GlobalCheckpoint {
    pub totalDelegationCap: BigUint,
    pub finished:   bool,
    pub lastID:     u64,
}

impl Encode for GlobalCheckpoint {
    #[inline]
    fn dep_encode_to<O: Output>(&self, dest: &mut O) {
        self.totalDelegationCap.dep_encode_to(dest);
        self.finished.dep_encode_to(dest);
        self.lastID.dep_encode_to(dest);
    }
}

impl Decode for GlobalCheckpoint {
    #[inline]
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(GlobalCheckpoint{
            totalDelegationCap:  BigUint::dep_decode(input)?,
            finished:            bool::dep_decode(input)?,
            lastID:              u64::dep_decode(input)?,
        })
    }
}