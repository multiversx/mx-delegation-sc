use multiversx_sc::api::BigUintApi;
use multiversx_sc::codec::*;

multiversx_sc::derive_imports!();

#[derive(TypeAbi, PartialEq, Debug)]
pub struct FundsListInfo<BigUint: BigUintApi> {
    pub total_balance: BigUint,
    pub first: usize,
    pub last: usize,
}

impl<M: ManagedTypeApi> FundsListInfo<Self::Api> {
    pub fn is_empty(&self) -> bool {
        self.total_balance == 0 && self.first == 0 && self.last == 0
    }

    pub fn zero_value() -> Self {
        FundsListInfo {
            total_balance: BigUint::zero(),
            first: 0,
            last: 0,
        }
    }
}

impl<M: ManagedTypeApi> NestedEncode for FundsListInfo<Self::Api> {
    fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.total_balance.dep_encode(dest)?;
        self.first.dep_encode(dest)?;
        self.last.dep_encode(dest)?;
        Ok(())
    }

    #[allow(clippy::redundant_clone)]
    fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
        &self,
        dest: &mut O,
        c: ExitCtx,
        exit: fn(ExitCtx, EncodeError) -> !,
    ) {
        self.total_balance.dep_encode_or_exit(dest, c.clone(), exit);
        self.first.dep_encode_or_exit(dest, c.clone(), exit);
        self.last.dep_encode_or_exit(dest, c.clone(), exit);
    }
}

impl<M: ManagedTypeApi> TopEncode for FundsListInfo<Self::Api> {
    #[inline]
    fn top_encode<O: TopEncodeOutput>(&self, output: O) -> Result<(), EncodeError> {
        // delete storage when the balance reaches 0
        // also require links to have been reset (this check is not strictly necessary, but improves safety)
        if self.is_empty() {
            output.set_slice_u8(&[]);
            Ok(())
        } else {
            top_encode_from_nested(self, output)
        }
    }

    #[inline]
    fn top_encode_or_exit<O: TopEncodeOutput, ExitCtx: Clone>(
        &self,
        output: O,
        c: ExitCtx,
        exit: fn(ExitCtx, EncodeError) -> !,
    ) {
        // delete storage when the balance reaches 0
        // also require links to have been reset (this check is not strictly necessary, but improves safety)
        if self.is_empty() {
            output.set_slice_u8(&[]);
        } else {
            top_encode_from_nested_or_exit(self, output, c, exit);
        }
    }
}

impl<M: ManagedTypeApi> NestedDecode for FundsListInfo<Self::Api> {
    fn dep_decode<I: NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
        Ok(FundsListInfo {
            total_balance: BigUint::dep_decode(input)?,
            first: usize::dep_decode(input)?,
            last: usize::dep_decode(input)?,
        })
    }

    #[allow(clippy::redundant_clone)]
    fn dep_decode_or_exit<I: NestedDecodeInput, ExitCtx: Clone>(
        input: &mut I,
        c: ExitCtx,
        exit: fn(ExitCtx, DecodeError) -> !,
    ) -> Self {
        FundsListInfo {
            total_balance: BigUint::dep_decode_or_exit(input, c.clone(), exit),
            first: usize::dep_decode_or_exit(input, c.clone(), exit),
            last: usize::dep_decode_or_exit(input, c.clone(), exit),
        }
    }
}

impl<M: ManagedTypeApi> TopDecode for FundsListInfo<Self::Api> {
    fn top_decode<I: TopDecodeInput>(input: I) -> Result<Self, DecodeError> {
        if input.byte_len() == 0 {
            // does not exist in storage
            Ok(FundsListInfo::zero_value())
        } else {
            top_decode_from_nested(input)
        }
    }

    fn top_decode_or_exit<I: TopDecodeInput, ExitCtx: Clone>(
        input: I,
        c: ExitCtx,
        exit: fn(ExitCtx, DecodeError) -> !,
    ) -> Self {
        if input.byte_len() == 0 {
            // does not exist in storage
            FundsListInfo::zero_value()
        } else {
            top_decode_from_nested_or_exit(input, c, exit)
        }
    }
}
