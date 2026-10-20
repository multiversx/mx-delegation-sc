use multiversx_sc_snippets::imports::*;

use crate::{
    latest_proxy,
    ops::{DelegationOps, StakeByType},
};

/// Queries. All of these are free - no transaction, no block, no fee - which is what
/// makes it affordable for the fuzzer to check state after every single operation.
impl DelegationOps<'_> {
    pub async fn version(&mut self) -> String {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .version()
            .returns(ReturnsResultAs::<String>::new())
            .run()
            .await
    }

    pub async fn total_stake_by_type(&mut self) -> StakeByType {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_total_stake_by_type_endpoint()
            .returns(ReturnsResult)
            .run()
            .await
            .into_tuple()
    }

    pub async fn user_stake_by_type(&mut self, user: &Bech32Address) -> StakeByType {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_user_stake_by_type_endpoint(user)
            .returns(ReturnsResult)
            .run()
            .await
            .into_tuple()
    }

    pub async fn all_user_stake_by_type(&mut self) -> Vec<(Bech32Address, StakeByType)> {
        let result = self
            .interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_all_user_stake_by_type()
            .returns(ReturnsResult)
            .run()
            .await;

        result
            .into_iter()
            .map(|entry| {
                let (address, stake_by_type) = entry.into_tuple();
                (
                    Bech32Address::from(address.to_address()),
                    stake_by_type.into_tuple(),
                )
            })
            .collect()
    }

    pub async fn service_fee(&mut self) -> RustBigUint {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_service_fee()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await
    }

    pub async fn owner_min_stake_share(&mut self) -> RustBigUint {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_owner_min_stake_share()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await
    }

    pub async fn num_users(&mut self) -> usize {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_num_users()
            .returns(ReturnsResult)
            .run()
            .await
    }

    pub async fn total_delegation_cap(&mut self) -> BigUint<StaticApi> {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_total_delegation_cap()
            .returns(ReturnsResult)
            .run()
            .await
    }

    pub async fn auction_contract_address(&mut self) -> Bech32Address {
        let address = self
            .interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_auction_contract_address()
            .returns(ReturnsResult)
            .run()
            .await;

        Bech32Address::from(address.to_address())
    }

    pub async fn is_bootstrap_mode(&mut self) -> bool {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .is_bootstrap_mode()
            .returns(ReturnsResult)
            .run()
            .await
    }

    pub async fn n_blocks_before_unbond(&mut self) -> u64 {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_n_blocks_before_unbond()
            .returns(ReturnsResult)
            .run()
            .await
    }

    pub async fn minimum_stake(&mut self) -> BigUint<StaticApi> {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_minimum_stake()
            .returns(ReturnsResult)
            .run()
            .await
    }

    pub async fn unstakeable(&mut self, user: &Bech32Address) -> BigUint<StaticApi> {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_unstakeable(user)
            .returns(ReturnsResult)
            .run()
            .await
    }

    pub async fn unbondable(&mut self, user: &Bech32Address) -> BigUint<StaticApi> {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_unbondable(user)
            .returns(ReturnsResult)
            .run()
            .await
    }

    /// Each entry is (amount, block nonce when it was registered).
    pub async fn user_deferred_payment_list(
        &mut self,
        user: &Bech32Address,
    ) -> Vec<(BigUint<StaticApi>, u64)> {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_user_deferred_payment_list(user)
            .returns(ReturnsResult)
            .run()
            .await
            .into_iter()
            .map(|pair| pair.into_tuple())
            .collect()
    }

    pub async fn voting_power(&mut self, user: &Bech32Address) -> BigUint<StaticApi> {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_voting_power(user)
            .returns(ReturnsResult)
            .run()
            .await
    }

    /// All validator rewards ever received by the contract. Monotonically increasing;
    /// this is what individual reward shares are computed against.
    pub async fn total_cumulated_rewards(&mut self) -> BigUint<StaticApi> {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_total_cumulated_rewards()
            .returns(ReturnsResult)
            .run()
            .await
    }

    pub async fn claimable_rewards(&mut self, user: &Bech32Address) -> BigUint<StaticApi> {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .get_claimable_rewards(user)
            .returns(ReturnsResult)
            .run()
            .await
    }

    pub async fn is_global_op_in_progress(&mut self) -> bool {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .is_global_op_in_progress()
            .returns(ReturnsResult)
            .run()
            .await
    }

    pub async fn global_op_checkpoint(
        &mut self,
    ) -> latest_proxy::GlobalOpCheckpoint<StaticApi> {
        self.interactor
            .query()
            .to(self.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .global_op_checkpoint()
            .returns(ReturnsResult)
            .run()
            .await
    }
}
