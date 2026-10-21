//! The fuzzer: a seeded sequence of random operations against a freshly deployed
//! contract, checking the contract's own invariants as it goes.
//!
//! The port of `fuzzDelegation_test.go`.

mod bounds;
mod events;
mod fuzz_runner;
mod log;
mod roulette;
mod state;

pub use events::Event;
pub use fuzz_runner::FuzzRunner;
