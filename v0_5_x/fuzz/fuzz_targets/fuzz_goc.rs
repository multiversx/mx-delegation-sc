#![no_main]
use libfuzzer_sys::fuzz_target;

use elrond_wasm::elrond_codec::*;
use elrond_wasm::elrond_codec::test_util::*;
use delegation_v0_5_x::reset_checkpoint_types::*;
use elrond_wasm_debug::RustBigUint;

fuzz_target!(|data: &[u8]| {
    if let Ok(decoded) = GlobalOpCheckpoint::<RustBigUint>::top_decode(data) {
        let encoded_clean = check_top_encode(&decoded);
        let decoded_again = check_top_decode::<GlobalOpCheckpoint<RustBigUint>>(&encoded_clean[..]);
        assert_eq!(decoded, decoded_again);
        let encoded_again = check_top_encode(&decoded_again);
        assert_eq!(encoded_clean, encoded_again);
    }
});
