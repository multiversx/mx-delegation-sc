pub mod bls_key;
pub mod bls_sig;
pub mod node_state;

pub use bls_key::BLSKey;
pub use bls_sig::BLSSignature;
pub use node_state::*;

pub type BLSStatusMultiArg = elrond_wasm::types::MultiArg2<BLSKey, i32>;
