use multiversx_sc::{api::ManagedTypeApi, types::ManagedByteArray};

multiversx_sc::derive_imports!();

// BLS keys have 96 bytes
pub const BLS_KEY_BYTE_LENGTH: usize = 96;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, ManagedVecItem, Clone)]
pub struct BLSKey<M: ManagedTypeApi> {
    pub bytes: ManagedByteArray<M, BLS_KEY_BYTE_LENGTH>,
}
