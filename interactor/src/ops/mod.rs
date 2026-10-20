//! Endpoint calls and queries against a delegation contract.
//!
//! These are deliberately independent of where the addresses came from, so that the
//! real-chain commands (driven by `config.toml`) and the chain-simulator harness
//! (driven by `test.toml`) share one implementation.

mod admin;
mod user;
mod views;

pub use admin::deploy;

use multiversx_sc_snippets::imports::*;

use crate::deleg_interactor_config::GasConfig;

/// Everything needed to call a delegation contract: the connection, the contract, and
/// the gas limits to use.
pub struct DelegationOps<'a> {
    pub interactor: &'a mut Interactor,
    pub sc_address: &'a Bech32Address,
    pub gas: &'a GasConfig,
}

impl<'a> DelegationOps<'a> {
    pub fn new(
        interactor: &'a mut Interactor,
        sc_address: &'a Bech32Address,
        gas: &'a GasConfig,
    ) -> Self {
        DelegationOps {
            interactor,
            sc_address,
            gas,
        }
    }
}

/// The contract's five fund buckets, in the order the contract returns them:
/// WithdrawOnly, Waiting, Active, UnStaked, DeferredPayment.
pub type StakeByType = (
    RustBigUint,
    RustBigUint,
    RustBigUint,
    RustBigUint,
    RustBigUint,
);

/// Renders an `OperationCompletionStatus` the way the contract's own tests name it.
pub fn operation_completion_status(status: OperationCompletionStatus) -> &'static str {
    match status {
        OperationCompletionStatus::Completed => "completed",
        OperationCompletionStatus::InterruptedBeforeOutOfGas => "interrupted",
    }
}
