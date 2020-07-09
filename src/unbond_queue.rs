use elrond_wasm::elrond_codec::*;

pub struct UnbondQueueItem<BigUint>
where BigUint: Encode + Decode
{
    pub user_id: usize,
    pub amount: BigUint,
}

impl<BigUint> Encode for UnbondQueueItem<BigUint>
where BigUint: Encode + Decode
{
    fn dep_encode_to<O: Output>(&self, dest: &mut O) {
        self.user_id.dep_encode_to(dest);
        self.amount.dep_encode_to(dest);
    }
}

impl<BigUint> Decode for UnbondQueueItem<BigUint>
where BigUint: Encode + Decode
{
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(UnbondQueueItem{
            user_id: usize::dep_decode(input)?,
            amount: BigUint::dep_decode(input)?,
        })
    }
}
