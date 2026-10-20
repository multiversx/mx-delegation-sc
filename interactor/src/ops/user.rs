use multiversx_sc_snippets::imports::*;

use crate::{latest_proxy, ops::DelegationOps};

impl DelegationOps<'_> {
    /// Sends EGLD into the contract, landing as active or waiting stake.
    pub async fn stake(&mut self, sender: &Bech32Address, amount: &RustBigUint) {
        self.interactor
            .tx()
            .from(sender)
            .to(self.sc_address)
            .gas(self.gas.stake)
            .typed(latest_proxy::DelegationFullProxy)
            .stake_endpoint()
            .egld(BigUint::<StaticApi>::from(amount))
            .run()
            .await;
    }

    /// Moves stake out of the active/waiting buckets, towards deferred payment.
    pub async fn unstake(&mut self, sender: &Bech32Address, amount: &RustBigUint) {
        self.interactor
            .tx()
            .from(sender)
            .to(self.sc_address)
            .gas(self.gas.unstake)
            .typed(latest_proxy::DelegationFullProxy)
            .unstake_endpoint(amount.clone())
            .run()
            .await;
    }

    /// Pays out whatever rewards the caller has accumulated.
    pub async fn claim_rewards(&mut self, sender: &Bech32Address) {
        self.interactor
            .tx()
            .from(sender)
            .to(self.sc_address)
            .gas(self.gas.claim_rewards)
            .typed(latest_proxy::DelegationFullProxy)
            .claim_rewards()
            .run()
            .await;
    }

    /// Withdraws whatever is currently unbondable, and returns how much that was.
    ///
    /// One call only: the endpoint works in bounded chunks, so a caller wanting everything
    /// keeps calling while `unbondable` is above zero.
    pub async fn unbond_user(&mut self, sender: &Bech32Address) -> RustBigUint {
        self.interactor
            .tx()
            .from(sender)
            .to(self.sc_address)
            .gas(self.gas.unbond)
            .typed(latest_proxy::DelegationFullProxy)
            .unbond_user()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await
    }

    /// Casts a governance vote with the caller's voting power.
    pub async fn delegate_vote(&mut self, sender: &Bech32Address, proposal: u64, vote: &str) {
        self.interactor
            .tx()
            .from(sender)
            .to(self.sc_address)
            .gas(self.gas.vote)
            .typed(latest_proxy::DelegationFullProxy)
            .delegate_vote(proposal, vote)
            .run()
            .await;
    }
}
