
use elrond_wasm::elrond_codec::*;
use elrond_wasm_debug::*;
use sc_delegation_rs::unbond_queue::*;
use elrond_wasm::Vec;

#[test]
fn test_unbond_queue_serialization() {
    let item = UnbondQueueItem {
        user_id: 5,
        amount: RustBigUint::from(7usize),
    };
    let expected_bytes: &[u8] = &[0, 0, 0, 5, 0, 0, 0, 1, 7];

    // serialize
    let serialized_bytes = item.top_encode();
    assert_eq!(serialized_bytes.as_slice(), expected_bytes);

    // deserialize
    let deserialized: UnbondQueueItem<RustBigUint> = decode_from_byte_slice(&serialized_bytes[..]).unwrap();
    assert_eq!(deserialized.user_id, item.user_id);
    assert_eq!(deserialized.amount, item.amount);
}

#[test]
fn test_opt_unbond_queue_serialization() {
    let item = UnbondQueueItem {
        user_id: 5,
        amount: RustBigUint::from(256usize),
    };
    let expected_bytes: &[u8] = &[1, 0, 0, 0, 5, 0, 0, 0, 2, 1, 0];

    // serialize
    let serialized_bytes = Some(item).top_encode();
    assert_eq!(serialized_bytes.as_slice(), expected_bytes);

    // deserialize
    let opt_deserialized: Option<UnbondQueueItem<RustBigUint>> = decode_from_byte_slice(&serialized_bytes[..]).unwrap();
    if let Some(deserialized) = &opt_deserialized{
        assert_eq!(deserialized.user_id, 5);
        assert_eq!(deserialized.amount, RustBigUint::from(256usize));
    }
}
