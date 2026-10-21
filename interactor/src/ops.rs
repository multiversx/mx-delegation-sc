//! Endpoint calls and queries against a delegation contract.
//!
//! These are deliberately independent of where the addresses came from, so that the
//! real-chain commands (driven by `config.toml`) and the chain-simulator harness
//! (driven by `test.toml`) share one implementation.
//!
//! Endpoint calls hand back the framework's `Result<_, TxResponseStatus>`. Several of them
//! fail as a matter of course - unstaking from an address that never staked, removing a
//! node that is not inactive - so it is the caller that decides which refusals are
//! ordinary and which are a finding.

mod admin;
mod delegation_ops;
mod dust;
mod invariants;
mod nodes;
mod user;
mod views;

pub use admin::deploy;
pub use delegation_ops::DelegationOps;
pub use nodes::NODE_STATE_INACTIVE;

use multiversx_sc_snippets::imports::*;

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
