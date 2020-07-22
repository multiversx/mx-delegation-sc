
use elrond_wasm::serializer::{to_bytes, from_bytes};
use sc_delegation_rs::bls_key::*;
use elrond_wasm::Vec;

#[test]
fn test_bls_serde() {
    let bls_key = BLSKey([4u8; BLS_KEY_BYTE_LENGTH]);
    let expected_bytes: &[u8] = &[4u8; BLS_KEY_BYTE_LENGTH];

    // serialize
    let serialized_bytes = to_bytes(&bls_key).unwrap();
    assert_eq!(serialized_bytes.as_slice(), expected_bytes);

    // deserialize
    let deserialized: BLSKey = from_bytes(serialized_bytes.as_slice()).unwrap();
    assert_eq!(deserialized.to_vec(), bls_key.to_vec());
}

#[test]
fn test_vec_bls_serde() {
    let mut bls_vec: Vec<BLSKey> = Vec::new();
    for _ in 0..3 {
        bls_vec.push(BLSKey([4u8; BLS_KEY_BYTE_LENGTH]));
    }
    let expected_bytes: &[u8] = &[4u8; BLS_KEY_BYTE_LENGTH*3];

    // serialize
    let serialized_bytes = to_bytes(&bls_vec).unwrap();
    assert_eq!(serialized_bytes.as_slice(), expected_bytes);

    // deserialize
    let deserialized: Vec<BLSKey> = from_bytes(serialized_bytes.as_slice()).unwrap();
    assert_eq!(deserialized.len(), bls_vec.len());
    for i in 0..3 {
        assert_eq!(deserialized[i].to_vec(), bls_vec[i].to_vec());
    }
}
