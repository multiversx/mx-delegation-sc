use elrond_wasm::{api::ManagedTypeApi, types::ManagedByteArray};

elrond_wasm::derive_imports!();

/// BLS signatures have 48 bytes
pub const BLS_SIGNATURE_BYTE_LENGTH: usize = 48;

// pub type BLSSignature<M> = ;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, ManagedVecItem, TypeAbi)]
pub struct BLSSignature<M: ManagedTypeApi> {
    pub bytes: ManagedByteArray<M, BLS_SIGNATURE_BYTE_LENGTH>,
}

// impl BLSSignature {
//     pub fn to_vec(&self) -> Vec<u8> {
//         self.0.to_vec()
//     }

//     pub fn from_array(arr: [u8; BLS_SIGNATURE_BYTE_LENGTH]) -> Self {
//         BLSSignature(Box::new(arr))
//     }
// }

// // only needed for tests
// use core::fmt;
// impl fmt::Debug for BLSSignature {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         self.0.fmt(f)
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use elrond_wasm::elrond_codec::test_util::*;
//     use elrond_wasm::Vec;

//     #[test]
//     fn test_bls_serialization() {
//         let bls_sig = BLSSignature::from_array([4u8; BLS_SIGNATURE_BYTE_LENGTH]);
//         let expected_bytes: &[u8] = &[4u8; BLS_SIGNATURE_BYTE_LENGTH];

//         // serialize
//         let serialized_bytes = check_top_encode(&bls_sig);
//         assert_eq!(serialized_bytes.as_slice(), expected_bytes);

//         // deserialize
//         let deserialized: BLSSignature = check_top_decode::<BLSSignature>(&serialized_bytes[..]);
//         assert_eq!(deserialized.to_vec(), bls_sig.to_vec());
//     }

//     #[test]
//     fn test_vec_bls_serialization() {
//         let mut bls_vec: Vec<BLSSignature> = Vec::new();
//         for _ in 0..3 {
//             bls_vec.push(BLSSignature::from_array([4u8; BLS_SIGNATURE_BYTE_LENGTH]));
//         }
//         let expected_bytes: &[u8] = &[4u8; BLS_SIGNATURE_BYTE_LENGTH * 3];

//         // serialize
//         let serialized_bytes = check_top_encode(&bls_vec);
//         assert_eq!(serialized_bytes.as_slice(), expected_bytes);

//         // deserialize
//         let deserialized: Vec<BLSSignature> =
//             check_top_decode::<Vec<BLSSignature>>(&serialized_bytes[..]);
//         assert_eq!(deserialized.len(), bls_vec.len());
//         for i in 0..3 {
//             assert_eq!(deserialized[i].to_vec(), bls_vec[i].to_vec());
//         }
//     }
// }
