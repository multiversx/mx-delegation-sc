use multiversx_sc::proxy_imports::*;

/// Temporary copy of framework proxy.
pub struct GovernanceSCLocalProxy;

impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for GovernanceSCLocalProxy
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    type TxProxyMethods = GovernanceSCLocalProxyMethods<Env, From, To, Gas>;

    fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
        GovernanceSCLocalProxyMethods { wrapped_tx: tx }
    }
}

/// Method container of the Governance system smart contract proxy.
pub struct GovernanceSCLocalProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
}

impl<Env, From, To, Gas> GovernanceSCLocalProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn delegate_vote<
        Arg0: ProxyArg<u64>,
        Arg1: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg2: ProxyArg<ManagedAddress<Env::Api>>,
        Arg3: ProxyArg<BigUint<Env::Api>>,
    >(
        self,
        proposal_to_vote: Arg0,
        vote: Arg1,
        voter: Arg2,
        user_stake: Arg3,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("delegateVote")
            .argument(&proposal_to_vote)
            .argument(&vote)
            .argument(&voter)
            .argument(&user_stake)
            .original_result()
    }
}
