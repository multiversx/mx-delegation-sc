#![no_main]
use libfuzzer_sys::fuzz_target;

use elrond_wasm::elrond_codec::*;
use user_fund_storage::types::*;
use elrond_wasm_debug::*;

fuzz_target!(|data: &[u8]| {
    if let Ok(decoded) = FundsListInfo::<RustBigUint>::top_decode(&mut &data[..]) {
        let encoded_clean = decoded.top_encode().unwrap();
        let decoded_again = FundsListInfo::<RustBigUint>::top_decode(&mut &encoded_clean[..]).unwrap();
        assert_eq!(decoded, decoded_again);
        let encoded_again = decoded_again.top_encode().unwrap();
        assert_eq!(encoded_clean, encoded_again);
    }
});