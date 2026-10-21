//! Endpoint calls and queries against a delegation contract.
//!
//! These are deliberately independent of where the addresses came from, so that the
//! real-chain commands (driven by `config.toml`) and the chain-simulator harness
//! (driven by `test.toml`) share one implementation.

mod admin;
mod dust;
mod invariants;
mod nodes;
mod user;
mod views;

pub use admin::deploy;
pub use nodes::NODE_STATE_INACTIVE;

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

/// Why a transaction was rejected.
///
/// Several operations fail as a matter of course - unstaking from an address that never
/// staked, removing a node that is not inactive - so the fuzzer has to tell an ordinary
/// rejection apart from a real finding. Running out of gas is the case the Go fuzzer
/// singled out: it means the endpoint cannot finish within its budget, which is a defect
/// rather than a business rule.
#[derive(Debug)]
pub struct OpError {
    pub status: ReturnCode,
    pub message: String,
}

impl OpError {
    pub fn out_of_gas(&self) -> bool {
        self.status == ReturnCode::OutOfGas
    }
}

impl core::fmt::Display for OpError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<TxResponseStatus> for OpError {
    fn from(status: TxResponseStatus) -> Self {
        OpError {
            status: status.status,
            message: status.message,
        }
    }
}

/// The result of an operation that the contract may legitimately refuse.
pub type OpResult<T = ()> = Result<T, OpError>;

/// Renders an `OperationCompletionStatus` the way the contract's own tests name it.
pub fn operation_completion_status(status: OperationCompletionStatus) -> &'static str {
    match status {
        OperationCompletionStatus::Completed => "completed",
        OperationCompletionStatus::InterruptedBeforeOutOfGas => "interrupted",
    }
}
