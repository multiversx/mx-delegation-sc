use multiversx_sc::api::BigUintApi;
use multiversx_sc::codec::*;

use super::fund_type::FundDescription;

multiversx_sc::derive_imports!();

/// A unit of balance, usually stake.
/// Contains a description of the source/intent of the funds, together with a balance.
#[type_abi]
#[derive(PartialEq, Debug)]
pub struct FundItem<BigUint: BigUintApi> {
    pub fund_desc: FundDescription,
    pub user_id: usize,
    pub balance: BigUint,
    pub type_list_next: usize,
    pub type_list_prev: usize,
    pub user_list_next: usize,
    pub user_list_prev: usize,
}

impl<M: ManagedTypeApi> FundItem<Self::Api> {
    pub fn is_zero_value(&self) -> bool {
        self.balance == 0
            && self.type_list_next == 0
            && self.type_list_prev == 0
            && self.user_list_next == 0
            && self.user_list_prev == 0
    }

    pub fn zero_value() -> Self {
        FundItem {
            fund_desc: FundDescription::WithdrawOnly,
            user_id: 0,
            balance: BigUint::zero(),
            type_list_next: 0,
            type_list_prev: 0,
            user_list_next: 0,
            user_list_prev: 0,
        }
    }
}

impl<M: ManagedTypeApi> NestedEncode for FundItem<Self::Api> {
    fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
        self.fund_desc.dep_encode(dest)?;
        self.user_id.dep_encode(dest)?;
        self.balance.dep_encode(dest)?;
        self.type_list_next.dep_encode(dest)?;
        self.type_list_prev.dep_encode(dest)?;
        self.user_list_next.dep_encode(dest)?;
        self.user_list_prev.dep_encode(dest)?;
        Ok(())
    }

    #[allow(clippy::redundant_clone)]
    fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
        &self,
        dest: &mut O,
        c: ExitCtx,
        exit: fn(ExitCtx, EncodeError) -> !,
    ) {
        self.fund_desc.dep_encode_or_exit(dest, c.clone(), exit);
        self.user_id.dep_encode_or_exit(dest, c.clone(), exit);
        self.balance.dep_encode_or_exit(dest, c.clone(), exit);
        self.type_list_next
            .dep_encode_or_exit(dest, c.clone(), exit);
        self.type_list_prev
            .dep_encode_or_exit(dest, c.clone(), exit);
        self.user_list_next
            .dep_encode_or_exit(dest, c.clone(), exit);
        self.user_list_prev
            .dep_encode_or_exit(dest, c.clone(), exit);
    }
}

impl<M: ManagedTypeApi> TopEncode for FundItem<Self::Api> {
    #[inline]
    fn top_encode<O: TopEncodeOutput>(&self, output: O) -> Result<(), EncodeError> {
        // delete storage when the balance reaches 0
        // also require links to have been reset (this check is not strictly necessary, but improves safety)
        if self.is_zero_value() {
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
        if self.is_zero_value() {
            output.set_slice_u8(&[]);
        } else {
            top_encode_from_nested_or_exit(self, output, c, exit);
        }
    }
}

impl<M: ManagedTypeApi> NestedDecode for FundItem<Self::Api> {
    fn dep_decode<I: NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
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

    #[allow(clippy::redundant_clone)]
    fn dep_decode_or_exit<I: NestedDecodeInput, ExitCtx: Clone>(
        input: &mut I,
        c: ExitCtx,
        exit: fn(ExitCtx, DecodeError) -> !,
    ) -> Self {
        FundItem {
            fund_desc: FundDescription::dep_decode_or_exit(input, c.clone(), exit),
            user_id: usize::dep_decode_or_exit(input, c.clone(), exit),
            balance: BigUint::dep_decode_or_exit(input, c.clone(), exit),
            type_list_next: usize::dep_decode_or_exit(input, c.clone(), exit),
            type_list_prev: usize::dep_decode_or_exit(input, c.clone(), exit),
            user_list_next: usize::dep_decode_or_exit(input, c.clone(), exit),
            user_list_prev: usize::dep_decode_or_exit(input, c.clone(), exit),
        }
    }
}

impl<M: ManagedTypeApi> TopDecode for FundItem<Self::Api> {
    fn top_decode<I: TopDecodeInput>(input: I) -> Result<Self, DecodeError> {
        if input.byte_len() == 0 {
            // does not exist in storage
            Ok(FundItem::zero_value())
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
            FundItem::zero_value()
        } else {
            top_decode_from_nested_or_exit(input, c, exit)
        }
    }
}
