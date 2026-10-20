//! Preparing a chain-simulator environment to fuzz against: deploy a fresh contract,
//! bring the delegators and the sinks into existence, and register them as senders.
//!
//! The port of `fuzzDelegationInit.go`, minus the parts the chain simulator provides by
//! itself - there is no world to clear and no auction mock to deploy, since the real
//! system auction contract is already there.

use multiversx_sc_snippets::imports::*;

use crate::{
    display::display_egld_amount,
    ops,
    test_env::{TestAccounts, TestInteractor},
};

impl TestInteractor {
    /// Deploys a contract and prepares every account a fuzz run needs.
    ///
    /// Starts from scratch each time: the delegators get their configured balance and a
    /// zero nonce, and the sinks are emptied, so two runs cannot contaminate each other.
    pub async fn fuzz_setup(&mut self) {
        let owner = self.config.owner.address();

        let sc_address = ops::deploy(
            &mut self.interactor,
            &owner,
            &self.config.deploy,
            &self.config.gas,
        )
        .await;
        self.config.chain_config.sc_address = Some(sc_address.clone());

        let accounts = self.accounts();

        // Registering makes the interactor sign for the address - with an all-zero
        // signature, since these have no private key - and gives it the simulator's
        // standard funding, which the exact balances below then overwrite.
        for delegator in accounts.generated_delegators() {
            self.interactor.register_address(delegator.clone()).await;
        }

        self.fund(&accounts).await;
        self.report_setup(&sc_address, &accounts).await;
    }

    /// Sets exact balances: the configured amount for every generated delegator, and zero
    /// for the sinks, which must start empty for the closing invariants to mean anything.
    async fn fund(&mut self, accounts: &TestAccounts) {
        let balance = self.config.delegators.initial_balance();
        let empty = RustBigUint::default();

        let mut set_state_accounts: Vec<SetStateAccount> = accounts
            .generated_delegators()
            .iter()
            .map(|delegator| account_with_balance(delegator, &balance))
            .collect();

        set_state_accounts.extend(
            accounts
                .sinks()
                .into_iter()
                .map(|sink| account_with_balance(sink, &empty)),
        );

        self.interactor
            .set_state(set_state_accounts)
            .await
            .expect("could not set delegator and sink balances");
    }

    async fn report_setup(&mut self, sc_address: &Bech32Address, accounts: &TestAccounts) {
        let shard_config = self.shard_config();
        let balance = self.config.delegators.initial_balance();

        println!();
        println!(
            "Contract:  {sc_address}  (shard {})",
            shard_config.compute_id(sc_address.as_address())
        );
        println!("Owner:     {}", accounts.delegator(0));
        println!();

        println!(
            "{} delegators, {} each:",
            accounts.generated_delegators().len(),
            display_egld_amount(&balance)
        );
        for (index, delegator) in accounts.generated_delegators().iter().enumerate() {
            println!(
                "  {:>3}  shard {}  {delegator}",
                index + 1,
                shard_config.compute_id(delegator.as_address())
            );
        }

        println!();
        println!("Rewards sink:  {}", accounts.rewards_sink);
        println!("Withdraw sink: {}", accounts.withdraw_sink);
        println!();
        println!("Set sc_address in the config file to keep using this contract:");
        println!("  sc_address = \"{sc_address}\"");
    }
}

/// An account whose balance is set to an exact amount, leaving code and the rest alone.
///
/// The nonce goes to zero, which is what a fresh setup wants; note that this would reset
/// an account that had already sent transactions.
fn account_with_balance(address: &Bech32Address, balance: &RustBigUint) -> SetStateAccount {
    let mut account = SetStateAccount::from_address(address.to_bech32_string());
    account.balance = balance.to_string();
    account
}
