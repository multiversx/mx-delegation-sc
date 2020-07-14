use elrond_wasm::elrond_codec::*;

/// Pending payment for stake sale.
/// The seller cannot withdraw the payment immediately.
/// A StakeSalePayment object is saved in a queue, every time someone buys stake,
/// and only when enough time has passed can it be claimed.
pub struct StakeSalePayment<BigUint>
where BigUint: Encode + Decode
{
    pub user_id: usize,
    pub amount: BigUint,
    pub claim_after_nonce: u64,
}

impl<BigUint> Encode for StakeSalePayment<BigUint>
where BigUint: Encode + Decode
{
    fn dep_encode_to<O: Output>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.user_id.dep_encode_to(dest)?;
        self.amount.dep_encode_to(dest)?;
        self.claim_after_nonce.dep_encode_to(dest)?;
        Ok(())
    }
}

impl<BigUint> Decode for StakeSalePayment<BigUint>
where BigUint: Encode + Decode
{
    fn dep_decode<I: Input>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(StakeSalePayment{
            user_id: usize::dep_decode(input)?,
            amount: BigUint::dep_decode(input)?,
            claim_after_nonce: u64::dep_decode(input)?,
        })
    }
}
