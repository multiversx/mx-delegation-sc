//! The chain-simulator test harness: the fuzzer, and the manual actions that mirror its
//! operations.
//!
//! Driven by `test.toml`, see [`TestConfig`].

mod accounts;
mod actions;
mod config;
mod setup;

pub use accounts::TestAccounts;
pub use config::{DelegatorsConfig, EventWeights, TestConfig, parse_amount};

use multiversx_sc::chain_core::types::ShardConfig;
use multiversx_sc_snippets::imports::*;

use crate::ops::DelegationOps;

pub struct TestInteractor {
    pub(crate) interactor: Interactor,
    pub(crate) config: TestConfig,
}

impl TestInteractor {
    pub async fn init(config_file: &str) -> Self {
        let mut interactor = Interactor::empty().with_current_dir(env!("CARGO_MANIFEST_DIR"));
        let config: TestConfig = interactor.load_config_from_file(config_file).await;

        TestInteractor { interactor, config }
    }

    /// How the network splits addresses between shards, read from the network itself
    /// rather than assumed, so a simulator with a different shard count still works.
    fn shard_config(&self) -> ShardConfig {
        let num_shards = self.interactor.network_config().num_shards_without_meta;
        ShardConfig::new(num_shards)
            .unwrap_or_else(|| panic!("network reports an invalid shard count: {num_shards}"))
    }

    /// The delegators and sinks for this configuration. Derived, not stored, so every
    /// command agrees on them without a state file.
    pub(crate) fn accounts(&self) -> TestAccounts {
        TestAccounts::generate(
            self.config.owner.address(),
            &self.config.delegators,
            &self.shard_config(),
            self.interactor.get_hrp(),
        )
    }

    /// Who a user action acts as: a generated delegator when an index is given, otherwise
    /// the `[user]` wallet.
    ///
    /// Delegator 0 is the owner, which is already a registered signing wallet - it must
    /// not be re-registered as a signature-less address.
    async fn sender(&mut self, delegator: Option<usize>) -> Bech32Address {
        let Some(index) = delegator else {
            return self.config.user.address();
        };
        if index == 0 {
            return self.config.owner.address();
        }

        let address = self.accounts().delegator(index).clone();
        self.interactor.register_address(address.clone()).await;
        address
    }

    pub(crate) fn ops(&mut self) -> DelegationOps<'_> {
        DelegationOps::new(
            &mut self.interactor,
            self.config.chain_config.sc_address(),
            &self.config.gas,
        )
    }

    /// An account's EGLD balance, as a plain integer.
    pub(crate) async fn balance(&self, address: &Bech32Address) -> RustBigUint {
        let balance = self
            .interactor
            .get_account(address.as_address())
            .await
            .balance;
        if balance.is_empty() {
            return RustBigUint::default();
        }

        balance
            .parse()
            .unwrap_or_else(|_| panic!("unexpected balance value from gateway: '{balance}'"))
    }

    /// Moves an exact amount from a delegator to a sink.
    ///
    /// Here because on the chain simulator every transaction burns a real fee, so the Go
    /// fuzzer's way of proving rewards were paid out - summing delegator balances - no
    /// longer holds. Forwarding what a delegator receives to an account that never spends
    /// keeps that figure exact, and leaves the delegator's own balance free to absorb fees.
    pub(crate) async fn forward(
        &mut self,
        from: &Bech32Address,
        to: &Bech32Address,
        amount: &RustBigUint,
    ) {
        if amount == &RustBigUint::default() {
            return;
        }

        self.interactor
            .tx()
            .from(from)
            .to(to)
            .egld(BigUint::<StaticApi>::from(amount))
            .gas(self.config.gas.transfer)
            .run()
            .await;
    }
}
