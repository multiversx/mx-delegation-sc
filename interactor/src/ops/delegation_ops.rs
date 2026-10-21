//! The handle the endpoint calls and queries hang off.

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
