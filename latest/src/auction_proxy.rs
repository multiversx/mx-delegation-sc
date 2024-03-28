use multiversx_sc::proxy_imports::*;

/// Proxy for the auction system smart contract.
/// 
/// Initially auto-generated from the mock implementation, but cleaned up by hand.
/// For insance, because it is a system SC, one cannot call `init`` or `upgrade`` on it.
/// 
/// TODO: being a system SC, might be worth adding to the framework.
pub struct AuctionProxy;

impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for AuctionProxy
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    type TxProxyMethods = AuctionProxyMethods<Env, From, To, Gas>;

    fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
        AuctionProxyMethods { wrapped_tx: tx }
    }
}

pub struct AuctionProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
}

impl<Env, From, To, Gas> AuctionProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn stake<
        Arg0: CodecInto<usize>,
        Arg1: CodecInto<
            MultiValueEncoded<
                Env::Api,
                MultiValue2<
                    node_storage::types::BLSKey<Env::Api>,
                    node_storage::types::BLSSignature<Env::Api>,
                >,
            >,
        >,
    >(
        self,
        num_nodes: Arg0,
        bls_keys_signatures: Arg1,
    ) -> TxProxyCall<Env, From, To, Gas, MultiValueEncoded<Env::Api, ManagedBuffer<Env::Api>>> {
        self.wrapped_tx
            .raw_call()
            .function_name("stake")
            .argument(&num_nodes)
            .argument(&bls_keys_signatures)
            .original_result()
    }

    pub fn unstake<
        Arg0: CodecInto<MultiValueManagedVec<Env::Api, node_storage::types::BLSKey<Env::Api>>>,
    >(
        self,
        bls_keys: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, MultiValueEncoded<Env::Api, ManagedBuffer<Env::Api>>> {
        self.wrapped_tx
            .raw_call()
            .function_name("unStake")
            .argument(&bls_keys)
            .original_result()
    }

    pub fn unstake_nodes<
        Arg0: CodecInto<MultiValueManagedVec<Env::Api, node_storage::types::BLSKey<Env::Api>>>,
    >(
        self,
        bls_keys: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, MultiValueEncoded<Env::Api, ManagedBuffer<Env::Api>>> {
        self.wrapped_tx
            .raw_call()
            .function_name("unStakeNodes")
            .argument(&bls_keys)
            .original_result()
    }

    pub fn unbond<
        Arg0: CodecInto<MultiValueManagedVec<Env::Api, node_storage::types::BLSKey<Env::Api>>>,
    >(
        self,
        bls_keys: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, MultiValueEncoded<Env::Api, ManagedBuffer<Env::Api>>> {
        self.wrapped_tx
            .raw_call()
            .function_name("unBond")
            .argument(&bls_keys)
            .original_result()
    }

    pub fn unbond_nodes<
        Arg0: CodecInto<MultiValueManagedVec<Env::Api, node_storage::types::BLSKey<Env::Api>>>,
    >(
        self,
        bls_keys: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, MultiValueEncoded<Env::Api, ManagedBuffer<Env::Api>>> {
        self.wrapped_tx
            .raw_call()
            .function_name("unBondNodes")
            .argument(&bls_keys)
            .original_result()
    }

    pub fn unstake_tokens<Arg0: CodecInto<BigUint<Env::Api>>>(
        self,
        _amount: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call()
            .function_name("unStakeTokens")
            .argument(&_amount)
            .original_result()
    }

    pub fn unbond_tokens<Arg0: CodecInto<BigUint<Env::Api>>>(
        self,
        amount: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call()
            .function_name("unBondTokens")
            .argument(&amount)
            .original_result()
    }

    pub fn claim(self) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call()
            .function_name("claim")
            .original_result()
    }

    pub fn unjail<
        Arg0: CodecInto<MultiValueManagedVec<Env::Api, node_storage::types::BLSKey<Env::Api>>>,
    >(
        self,
        bls_keys: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call()
            .function_name("unJail")
            .argument(&bls_keys)
            .original_result()
    }
}
