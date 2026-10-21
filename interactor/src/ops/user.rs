use multiversx_sc_snippets::imports::*;

use crate::{
    latest_proxy,
    ops::{DelegationOps, OpResult},
};

impl DelegationOps<'_> {
    /// Sends EGLD into the contract, landing as active or waiting stake.
    pub async fn stake(&mut self, sender: &Bech32Address, amount: &RustBigUint) -> OpResult {
        self.interactor
            .tx()
            .from(sender)
            .to(self.sc_address)
            .gas(self.gas.stake)
            .typed(latest_proxy::DelegationFullProxy)
            .stake_endpoint()
            .egld(BigUint::<StaticApi>::from(amount))
            .returns(ReturnsHandledOrError::new())
            .run()
            .await
            .map_err(Into::into)
    }

    /// Moves stake out of the active/waiting buckets, towards deferred payment.
    pub async fn unstake(&mut self, sender: &Bech32Address, amount: &RustBigUint) -> OpResult {
        self.interactor
            .tx()
            .from(sender)
            .to(self.sc_address)
            .gas(self.gas.unstake)
            .typed(latest_proxy::DelegationFullProxy)
            .unstake_endpoint(amount.clone())
            .returns(ReturnsHandledOrError::new())
            .run()
            .await
            .map_err(Into::into)
    }

    /// Pays out whatever rewards the caller has accumulated.
    pub async fn claim_rewards(&mut self, sender: &Bech32Address) -> OpResult {
        self.interactor
            .tx()
            .from(sender)
            .to(self.sc_address)
            .gas(self.gas.claim_rewards)
            .typed(latest_proxy::DelegationFullProxy)
            .claim_rewards()
            .returns(ReturnsHandledOrError::new())
            .run()
            .await
            .map_err(Into::into)
    }

    /// Withdraws whatever is currently unbondable, and returns how much that was.
    ///
    /// One call only: the endpoint works in bounded chunks, so a caller wanting everything
    /// keeps calling while `unbondable` is above zero.
    pub async fn unbond_user(&mut self, sender: &Bech32Address) -> OpResult<RustBigUint> {
        self.interactor
            .tx()
            .from(sender)
            .to(self.sc_address)
            .gas(self.gas.unbond)
            .typed(latest_proxy::DelegationFullProxy)
            .unbond_user()
            .returns(ReturnsHandledOrError::new().returns(ReturnsResultUnmanaged))
            .run()
            .await
            .map_err(Into::into)
    }

    /// Casts a governance vote with the caller's voting power.
    pub async fn delegate_vote(
        &mut self,
        sender: &Bech32Address,
        proposal: u64,
        vote: &str,
    ) -> OpResult {
        self.interactor
            .tx()
            .from(sender)
            .to(self.sc_address)
            .gas(self.gas.vote)
            .typed(latest_proxy::DelegationFullProxy)
            .delegate_vote(proposal, vote)
            .returns(ReturnsHandledOrError::new())
            .run()
            .await
            .map_err(Into::into)
    }
}
