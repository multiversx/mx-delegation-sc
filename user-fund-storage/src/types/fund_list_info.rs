use elrond_wasm::api::BigUintApi;

elrond_wasm::derive_imports!();

#[derive(
    TopEncodeOrDefault, TopDecodeOrDefault, NestedEncode, NestedDecode, TypeAbi, PartialEq, Debug,
)]
pub struct FundsListInfo<BigUint: BigUintApi> {
    pub total_balance: BigUint,
    pub first: usize,
    pub last: usize,
}

impl<BigUint: BigUintApi> elrond_codec::EncodeDefault for FundsListInfo<BigUint> {
    fn is_default(&self) -> bool {
        self.total_balance == 0 && self.first == 0 && self.last == 0
    }
}

impl<BigUint: BigUintApi> elrond_codec::DecodeDefault for FundsListInfo<BigUint> {
    fn default() -> Self {
        FundsListInfo {
            total_balance: BigUint::zero(),
            first: 0,
            last: 0,
        }
    }
}
