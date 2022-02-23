use elrond_wasm::{api::ManagedTypeApi, types::ManagedByteArray};

elrond_wasm::derive_imports!();

// BLS keys have 96 bytes
pub const BLS_KEY_BYTE_LENGTH: usize = 96;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, ManagedVecItem, TypeAbi)]
pub struct BLSKey<M: ManagedTypeApi> {
    pub bytes: ManagedByteArray<M, BLS_KEY_BYTE_LENGTH>,
}
