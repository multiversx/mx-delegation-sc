//! The chain-simulator test harness: the fuzzer, and the manual actions that mirror its
//! operations.
//!
//! Driven by `test.toml`, see [`TestConfig`].

mod config;

pub use config::{TestConfig, parse_amount};

use multiversx_sc_snippets::imports::*;

use crate::{
    display::{display_egld_amount, display_egld_rust_amount},
    ops::{self, DelegationOps, operation_completion_status},
};

pub struct TestInteractor {
    interactor: Interactor,
    config: TestConfig,
}

impl TestInteractor {
    pub async fn init(config_file: &str) -> Self {
        let mut interactor = Interactor::empty().with_current_dir(env!("CARGO_MANIFEST_DIR"));
        let config: TestConfig = interactor.load_config_from_file(config_file).await;

        TestInteractor { interactor, config }
    }

    fn ops(&mut self) -> DelegationOps<'_> {
        DelegationOps::new(
            &mut self.interactor,
            self.config.chain_config.sc_address(),
            &self.config.gas,
        )
    }

    pub async fn deploy(&mut self) {
        let owner = self.config.owner.address();
        let new_address =
            ops::deploy(&mut self.interactor, &owner, &self.config.deploy, &self.config.gas).await;

        println!("New contract address: {new_address}");
        println!("Set it as sc_address in the config file to use it.");
    }

    pub async fn stake(&mut self, amount: Option<String>) {
        let user = self.config.user.address();
        let amount = match amount {
            Some(raw) => parse_amount(&raw, "amount"),
            None => self.config.stake_amount(),
        };

        println!("Staking {} from {user}", display_egld_amount(&amount));
        self.ops().stake(&user, amount).await;
        println!("Stake done");
    }

    pub async fn unstake(&mut self, amount: Option<String>) {
        let user = self.config.user.address();
        let amount = match amount {
            Some(raw) => parse_amount(&raw, "amount"),
            None => self.config.unstake_amount(),
        };

        println!("Unstaking {} from {user}", display_egld_amount(&amount));
        self.ops().unstake(&user, amount).await;
        println!("Unstake done");
    }

    pub async fn claim_rewards(&mut self) {
        let user = self.config.user.address();

        let claimable = self.ops().claimable_rewards(&user).await;
        println!("Claimable: {}", display_egld_amount(&claimable));

        self.ops().claim_rewards(&user).await;
        println!("Claim rewards done");
    }

    pub async fn delegate_vote(&mut self) {
        let user = self.config.user.address();
        let proposal = self.config.actions.vote_proposal;
        let decision = self.config.actions.vote_decision.clone();

        self.ops().delegate_vote(&user, proposal, &decision).await;
        println!("Vote sent");
    }

    pub async fn modify_delegation_cap(&mut self, amount: Option<String>) {
        let owner = self.config.owner.address();
        let new_cap = match amount {
            Some(raw) => parse_amount(&raw, "amount"),
            None => self.config.delegation_cap(),
        };

        println!("New delegation cap: {}", display_egld_amount(&new_cap));
        let status = self.ops().modify_total_delegation_cap(&owner, new_cap).await;

        println!(
            "Modified delegation cap: {}",
            operation_completion_status(status)
        );
    }

    /// Simulates a validator reward arriving at the contract.
    ///
    /// Chain simulator only: writes the protocol's own cumulated-reward storage key
    /// directly. See [`chain_sim::VALIDATOR_REWARD_KEY`].
    pub async fn add_rewards(&mut self, amount: Option<String>) {
        let sc_address = self.config.chain_config.sc_address().clone();
        let amount = match amount {
            Some(raw) => raw.parse().unwrap_or_else(|_| panic!("invalid amount: '{raw}'")),
            None => self.config.reward_amount(),
        };

        let before = self.ops().total_cumulated_rewards().await;
        let balance_before = self.balance(&sc_address).await;

        self.interactor
            .inject_validator_reward(&sc_address, &amount)
            .await
            .expect("could not inject a validator reward");

        let after = self.ops().total_cumulated_rewards().await;
        let balance_after = self.balance(&sc_address).await;

        println!("Injected reward:          {}", display_egld_rust_amount(&amount));
        println!("Cumulated rewards before: {}", display_egld_amount(&before));
        println!("Cumulated rewards after:  {}", display_egld_amount(&after));
        println!(
            "Contract balance:         {} -> {}",
            display_egld_rust_amount(&balance_before),
            display_egld_rust_amount(&balance_after)
        );

        let observed = &after - &before;
        let expected = BigUint::<StaticApi>::from_bytes_be(&amount.to_bytes_be());
        if observed == expected {
            println!("Contract sees the full reward.");
        } else {
            println!(
                "MISMATCH: contract saw {}, expected {}",
                display_egld_amount(&observed),
                display_egld_rust_amount(&amount)
            );
        }
    }

    pub async fn generate_blocks(&mut self, num_blocks: u64) {
        self.interactor
            .generate_blocks(num_blocks)
            .await
            .expect("could not generate blocks");

        println!("Generated {num_blocks} block(s).");
    }

    /// An account's EGLD balance, as a plain integer.
    async fn balance(&self, address: &Bech32Address) -> RustBigUint {
        let balance = self.interactor.get_account(address.as_address()).await.balance;
        if balance.is_empty() {
            return RustBigUint::default();
        }

        balance
            .parse()
            .unwrap_or_else(|_| panic!("unexpected balance value from gateway: '{balance}'"))
    }
}
