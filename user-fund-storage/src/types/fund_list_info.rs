use multiversx_sc::{api::ManagedTypeApi, types::BigUint};

multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncodeOrDefault, TopDecodeOrDefault, NestedEncode, NestedDecode, PartialEq, Debug)]
pub struct FundsListInfo<M: ManagedTypeApi> {
    pub total_balance: BigUint<M>,
    pub first: usize,
    pub last: usize,
}

impl<M: ManagedTypeApi> codec::EncodeDefault for FundsListInfo<M> {
    fn is_default(&self) -> bool {
        self.total_balance == 0 && self.first == 0 && self.last == 0
    }
}

impl<M: ManagedTypeApi> codec::DecodeDefault for FundsListInfo<M> {
    fn default() -> Self {
        FundsListInfo {
            total_balance: BigUint::zero(),
            first: 0,
            last: 0,
        }
    }
}
