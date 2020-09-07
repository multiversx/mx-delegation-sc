#![no_main]
use libfuzzer_sys::fuzz_target;

use elrond_wasm::elrond_codec::*;
use node_storage::types::*;

fuzz_target!(|data: &[u8]| {
    if let Ok(decoded) = BLSKey::top_decode(&mut &data[..]) {
        let encoded_clean = decoded.top_encode().unwrap();
        let decoded_again = BLSKey::top_decode(&mut &encoded_clean[..]).unwrap();
        let encoded_again = decoded_again.top_encode().unwrap();
        assert_eq!(encoded_clean,encoded_again);
    }
});
