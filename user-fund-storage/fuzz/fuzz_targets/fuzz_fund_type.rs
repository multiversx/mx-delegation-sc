#![no_main]
use libfuzzer_sys::fuzz_target;

use fuzz_util::check_encodings;
extern crate old_serialization;
use user_fund_storage::types as new_serialization;

fuzz_target!(|data: &[u8]| {
    check_encodings::<old_serialization::FundType, new_serialization::FundType>(data);
});
