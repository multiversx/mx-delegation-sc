use multiversx_sc::{api::ManagedTypeApi, types::ManagedByteArray};

multiversx_sc::derive_imports!();

/// BLS signatures have 48 bytes
pub const BLS_SIGNATURE_BYTE_LENGTH: usize = 48;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, ManagedVecItem)]
pub struct BLSSignature<M: ManagedTypeApi> {
    pub bytes: ManagedByteArray<M, BLS_SIGNATURE_BYTE_LENGTH>,
}
