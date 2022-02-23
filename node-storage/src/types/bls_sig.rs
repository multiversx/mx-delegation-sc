use elrond_wasm::{api::ManagedTypeApi, types::ManagedByteArray};

elrond_wasm::derive_imports!();

/// BLS signatures have 48 bytes
pub const BLS_SIGNATURE_BYTE_LENGTH: usize = 48;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, ManagedVecItem, TypeAbi)]
pub struct BLSSignature<M: ManagedTypeApi> {
    pub bytes: ManagedByteArray<M, BLS_SIGNATURE_BYTE_LENGTH>,
}
