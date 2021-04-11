use core::fmt::Debug;
use elrond_wasm::elrond_codec::test_util::{
    check_dep_decode, check_dep_encode, check_top_decode, check_top_encode,
};
use elrond_wasm::elrond_codec::{
    dep_decode_from_byte_slice, NestedDecode, NestedEncode, TopDecode, TopEncode,
};

fn check_eq(left: &[u8], right: &[u8]) {
    if left == &[0] && right == &[] {
        return;
    }
    assert_eq!(left, right);
}

fn check_top_encoding<OldType, NewType>(data: &[u8])
where
    OldType: TopEncode + TopDecode + PartialEq + Debug,
    NewType: TopEncode + TopDecode + PartialEq + Debug,
{
    if let Ok(decoded) = OldType::top_decode(Box::from(data)) {
        let encoded_clean = check_top_encode(&decoded);
        let decoded_again = check_top_decode::<NewType>(&encoded_clean[..]);
        let encoded_again = check_top_encode(&decoded_again);
        check_eq(&encoded_clean, &encoded_again);
    }
}

fn check_nested_encoding<OldType, NewType>(data: &[u8])
where
    OldType: NestedEncode + NestedDecode + PartialEq + Debug,
    NewType: NestedEncode + NestedDecode + PartialEq + Debug,
{
    if let Ok(decoded) = dep_decode_from_byte_slice::<OldType>(data) {
        let encoded_clean = check_dep_encode(&decoded);
        let decoded_again = check_dep_decode::<NewType>(&encoded_clean[..]);
        let encoded_again = check_dep_encode(&decoded_again);
        check_eq(&encoded_clean, &encoded_again);
    }
}

pub fn check_encodings<OldType, NewType>(data: &[u8])
where
    OldType: TopEncode + TopDecode + NestedEncode + NestedDecode + PartialEq + Debug,
    NewType: TopEncode + TopDecode + NestedEncode + NestedDecode + PartialEq + Debug,
{
    check_top_encoding::<OldType, NewType>(data);
    check_nested_encoding::<OldType, NewType>(data);
}
