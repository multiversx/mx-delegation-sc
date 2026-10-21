use multiversx_sc_snippets::imports::*;

use crate::{latest_proxy, ops::DelegationOps};

/// Dust cleanup sweeps fund entries too small to be worth their storage.
///
/// Both endpoints work in bounded chunks, so the caller keeps calling until the
/// corresponding count reaches zero.
impl DelegationOps<'_> {
    pub async fn count_dust_items_waiting_list(&mut self, dust_limit: &RustBigUint) -> usize {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .count_dust_items_waiting_list(dust_limit.clone())
            .returns(ReturnsResult)
            .run()
            .await
    }

    pub async fn count_dust_items_active(&mut self, dust_limit: &RustBigUint) -> usize {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .count_dust_items_active(dust_limit.clone())
            .returns(ReturnsResult)
            .run()
            .await
    }

    pub async fn dust_cleanup_waiting_list(
        &mut self,
        owner: &Bech32Address,
        dust_limit: &RustBigUint,
    ) -> Result<(), TxResponseStatus> {
        self.interactor
            .tx()
            .from(owner)
            .to(self.sc_address)
            .gas(self.gas.dust_cleanup)
            .typed(latest_proxy::DelegationFullProxy)
            .dust_cleanup_waiting_list(dust_limit.clone())
            .returns(ReturnsHandledOrError::new())
            .run()
            .await
    }

    pub async fn dust_cleanup_active(
        &mut self,
        owner: &Bech32Address,
        dust_limit: &RustBigUint,
    ) -> Result<(), TxResponseStatus> {
        self.interactor
            .tx()
            .from(owner)
            .to(self.sc_address)
            .gas(self.gas.dust_cleanup)
            .typed(latest_proxy::DelegationFullProxy)
            .dust_cleanup_active(dust_limit.clone())
            .returns(ReturnsHandledOrError::new())
            .run()
            .await
    }
}
