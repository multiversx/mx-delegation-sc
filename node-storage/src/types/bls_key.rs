use elrond_wasm::{Box, Vec};

elrond_wasm::derive_imports!();

// BLS keys have 96 bytes
pub const BLS_KEY_BYTE_LENGTH: usize = 96;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi)]
pub struct BLSKey(pub Box<[u8; BLS_KEY_BYTE_LENGTH]>);

impl BLSKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn from_array(arr: [u8; BLS_KEY_BYTE_LENGTH]) -> Self {
        BLSKey(Box::new(arr))
    }
}

// only needed for tests
use core::fmt;
impl fmt::Debug for BLSKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elrond_wasm::elrond_codec::test_util::*;
    use elrond_wasm::Vec;

    #[test]
    fn test_bls_serialization() {
        let bls_key = BLSKey::from_array([4u8; BLS_KEY_BYTE_LENGTH]);
        let expected_bytes: &[u8] = &[4u8; BLS_KEY_BYTE_LENGTH];

        // serialize
        let serialized_bytes = check_top_encode(&bls_key);
        assert_eq!(serialized_bytes.as_slice(), expected_bytes);

        // deserialize
        let deserialized: BLSKey = check_top_decode::<BLSKey>(&serialized_bytes[..]);
        assert_eq!(deserialized.to_vec(), bls_key.to_vec());
    }

    #[test]
    fn test_vec_bls_serialization() {
        let mut bls_vec: Vec<BLSKey> = Vec::new();
        for _ in 0..3 {
            bls_vec.push(BLSKey::from_array([4u8; BLS_KEY_BYTE_LENGTH]));
        }
        let expected_bytes: &[u8] = &[4u8; BLS_KEY_BYTE_LENGTH * 3];

        // serialize
        let serialized_bytes = check_top_encode(&bls_vec);
        assert_eq!(serialized_bytes.as_slice(), expected_bytes);

        // deserialize
        let deserialized: Vec<BLSKey> = check_top_decode::<Vec<BLSKey>>(&serialized_bytes[..]);
        assert_eq!(deserialized.len(), bls_vec.len());
        for i in 0..3 {
            assert_eq!(deserialized[i].to_vec(), bls_vec[i].to_vec());
        }
    }
}
