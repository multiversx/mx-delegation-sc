use multiversx_sc_snippets::imports::*;

use crate::{latest_proxy, LegacyDelegationInteractor};

impl LegacyDelegationInteractor {
    pub async fn stake_endpoint(&mut self) {
        let user_address = self.interactor.register_wallet(test_wallets::alice()).await;
        let egld_amount = BigUint::<StaticApi>::from(2_000000000000000000u128);

        self.interactor
            .tx()
            .from(user_address)
            .to(&self.config.sc_address)
            .gas(300_000_000u64)
            .typed(latest_proxy::DelegationFullProxy)
            .stake_endpoint()
            .egld(egld_amount)
            .run()
            .await;

        println!("Stake done");
    }
}
