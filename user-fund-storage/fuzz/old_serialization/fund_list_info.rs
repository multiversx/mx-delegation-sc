use elrond_wasm::api::BigUintApi;
use elrond_wasm::elrond_codec::*;

elrond_wasm::derive_imports!();

#[derive(TypeAbi, PartialEq, Debug)]
pub struct FundsListInfo<BigUint: BigUintApi> {
    pub total_balance: BigUint,
    pub first: usize,
    pub last: usize,
}

impl<BigUint: BigUintApi> FundsListInfo<BigUint> {
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

impl<BigUint: BigUintApi> NestedEncode for FundsListInfo<BigUint> {
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

impl<BigUint: BigUintApi> TopEncode for FundsListInfo<BigUint> {
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

impl<BigUint: BigUintApi> NestedDecode for FundsListInfo<BigUint> {
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

impl<BigUint: BigUintApi> TopDecode for FundsListInfo<BigUint> {
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
