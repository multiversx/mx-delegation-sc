use multiversx_sc_snippets::imports::*;

use crate::{latest_proxy, ops::DelegationOps};

/// The contract's own consistency checks.
///
/// Both are endpoints that simply fail the transaction when the property is broken, so
/// the result is the verdict. The error is returned rather than panicked on, so a caller
/// can report every broken invariant instead of stopping at the first.
impl DelegationOps<'_> {
    /// The owner must hold at least their configured minimum share of the active stake.
    pub async fn validate_owner_stake_share(
        &mut self,
        sender: &Bech32Address,
    ) -> Result<(), String> {
        self.interactor
            .tx()
            .from(sender)
            .to(self.sc_address)
            .gas(self.gas.validate_invariant)
            .typed(latest_proxy::DelegationFullProxy)
            .validate_owner_stake_share()
            .returns(ReturnsHandledOrError::new())
            .run()
            .await
            .map_err(|err| err.message)
    }

    /// Active stake must not exceed the total delegation cap.
    pub async fn validate_delegation_cap_invariant(
        &mut self,
        sender: &Bech32Address,
    ) -> Result<(), String> {
        self.interactor
            .tx()
            .from(sender)
            .to(self.sc_address)
            .gas(self.gas.validate_invariant)
            .typed(latest_proxy::DelegationFullProxy)
            .validate_delegation_cap_invariant()
            .returns(ReturnsHandledOrError::new())
            .run()
            .await
            .map_err(|err| err.message)
    }
}
