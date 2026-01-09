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
            .gas(10_000_000u64)
            .typed(latest_proxy::DelegationFullProxy)
            .stake_endpoint()
            .egld(egld_amount)
            .run()
            .await;

        println!("Stake done");
    }

    pub async fn unstake_endpoint(&mut self) {
        let user_address = self
            .interactor
            .register_wallet(Wallet::from_pem_file("delegator1.pem").unwrap())
            .await;
        let egld_amount = BigUint::<StaticApi>::from(42000_000000000000000000u128);

        self.interactor
            .tx()
            .from(user_address)
            .to(&self.config.sc_address)
            .gas(30_000_000u64)
            .typed(latest_proxy::DelegationFullProxy)
            .unstake_endpoint(egld_amount)
            .run()
            .await;

        println!("Unstake done");
    }

    pub async fn claim_rewards(&mut self) {
        let user_address = self
            .interactor
            .register_wallet(Wallet::from_pem_file("delegator2.pem").unwrap())
            .await;

        self.interactor
            .tx()
            .from(user_address)
            .to(&self.config.sc_address)
            .gas(10_000_000u64)
            .typed(latest_proxy::DelegationFullProxy)
            .claim_rewards()
            .run()
            .await;

        println!("Claim rewards done");
    }

    pub async fn delegate_vote(&mut self) {
        let user_address = self
            .interactor
            .register_wallet(Wallet::from_pem_file("delegator1.pem").unwrap())
            .await;

        self.interactor
            .tx()
            .from(user_address)
            .to(&self.config.sc_address)
            .gas(100_000_000u64)
            .typed(latest_proxy::DelegationFullProxy)
            .delegate_vote(4u64, "yes")
            .run()
            .await;

        println!("Vote sent");
    }
}
