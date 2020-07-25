
use elrond_wasm::elrond_codec::*;
use node_storage::types::*;
use elrond_wasm::Vec;

#[test]
fn test_bls_serialization() {
    let bls_key = BLSKey::from_array([4u8; BLS_KEY_BYTE_LENGTH]);
    let expected_bytes: &[u8] = &[4u8; BLS_KEY_BYTE_LENGTH];

    // serialize
    let serialized_bytes = bls_key.top_encode().unwrap();
    assert_eq!(serialized_bytes.as_slice(), expected_bytes);

    // deserialize
    let deserialized: BLSKey = decode_from_byte_slice(&serialized_bytes[..]).unwrap();
    assert_eq!(deserialized.to_vec(), bls_key.to_vec());
}

#[test]
fn test_vec_bls_serialization() {
    let mut bls_vec: Vec<BLSKey> = Vec::new();
    for _ in 0..3 {
        bls_vec.push(BLSKey::from_array([4u8; BLS_KEY_BYTE_LENGTH]));
    }
    let expected_bytes: &[u8] = &[4u8; BLS_KEY_BYTE_LENGTH*3];

    // serialize
    let serialized_bytes = bls_vec.top_encode().unwrap();
    assert_eq!(serialized_bytes.as_slice(), expected_bytes);

    // deserialize
    let deserialized: Vec<BLSKey> = decode_from_byte_slice(serialized_bytes.as_slice()).unwrap();
    assert_eq!(deserialized.len(), bls_vec.len());
    for i in 0..3 {
        assert_eq!(deserialized[i].to_vec(), bls_vec[i].to_vec());
    }
}
