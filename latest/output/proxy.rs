// Code generated by the multiversx-sc proxy generator. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![allow(dead_code)]
#![allow(clippy::all)]

use multiversx_sc::proxy_imports::*;

pub struct DelegationFullProxy;

impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for DelegationFullProxy
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    type TxProxyMethods = DelegationFullProxyMethods<Env, From, To, Gas>;

    fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
        DelegationFullProxyMethods { wrapped_tx: tx }
    }
}

pub struct DelegationFullProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
}

#[rustfmt::skip]
impl<Env, From, Gas> DelegationFullProxyMethods<Env, From, (), Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    Gas: TxGas<Env>,
{
    /// This is the contract constructor, called only once when the contract is deployed. 
    pub fn init<
        Arg0: CodecInto<ManagedAddress<Env::Api>>,
        Arg1: CodecInto<usize>,
        Arg2: CodecInto<usize>,
        Arg3: CodecInto<u64>,
        Arg4: CodecInto<BigUint<Env::Api>>,
        Arg5: CodecInto<BigUint<Env::Api>>,
    >(
        self,
        auction_contract_addr: Arg0,
        service_fee_per_10000: Arg1,
        owner_min_stake_share_per_10000: Arg2,
        n_blocks_before_unbond: Arg3,
        minimum_stake: Arg4,
        total_delegation_cap: Arg5,
    ) -> TxProxyDeploy<Env, From, Gas, ()> {
        self.wrapped_tx
            .raw_deploy()
            .argument(&auction_contract_addr)
            .argument(&service_fee_per_10000)
            .argument(&owner_min_stake_share_per_10000)
            .argument(&n_blocks_before_unbond)
            .argument(&minimum_stake)
            .argument(&total_delegation_cap)
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> DelegationFullProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn upgrade(
        self,
    ) -> TxProxyUpgrade<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_upgrade()
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> DelegationFullProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn version(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, &'static str> {
        self.wrapped_tx
            .raw_call("version")
            .original_result()
    }

    /// The number of nodes that will run with the contract stake, as configured by the owner. 
    pub fn num_nodes(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, usize> {
        self.wrapped_tx
            .raw_call("getNumNodes")
            .original_result()
    }

    /// Each node gets a node id. This is in order to be able to iterate over their data. 
    /// This is a mapping from node BLS key to node id. 
    /// The key is the bytes "node_id" concatenated with the BLS key. The value is the node id. 
    /// Ids start from 1 because 0 means unset of None. 
    pub fn get_node_id<
        Arg0: CodecInto<node_storage::types::bls_key::BLSKey<Env::Api>>,
    >(
        self,
        bls_key: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, usize> {
        self.wrapped_tx
            .raw_call("getNodeId")
            .argument(&bls_key)
            .original_result()
    }

    pub fn get_node_signature_endpoint<
        Arg0: CodecInto<node_storage::types::bls_key::BLSKey<Env::Api>>,
    >(
        self,
        bls_key: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, OptionalValue<node_storage::types::bls_sig::BLSSignature<Env::Api>>> {
        self.wrapped_tx
            .raw_call("getNodeSignature")
            .argument(&bls_key)
            .original_result()
    }

    pub fn get_node_state_endpoint<
        Arg0: CodecInto<node_storage::types::bls_key::BLSKey<Env::Api>>,
    >(
        self,
        bls_key: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, node_storage::types::node_state::NodeState> {
        self.wrapped_tx
            .raw_call("getNodeState")
            .argument(&bls_key)
            .original_result()
    }

    pub fn get_all_node_states(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, MultiValueEncoded<Env::Api, MultiValue2<node_storage::types::bls_key::BLSKey<Env::Api>, u8>>> {
        self.wrapped_tx
            .raw_call("getAllNodeStates")
            .original_result()
    }

    pub fn get_node_bl_nonce_of_unstake_endpoint<
        Arg0: CodecInto<node_storage::types::bls_key::BLSKey<Env::Api>>,
    >(
        self,
        bls_key: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, OptionalValue<u64>> {
        self.wrapped_tx
            .raw_call("getNodeBlockNonceOfUnstake")
            .argument(&bls_key)
            .original_result()
    }

    pub fn add_nodes<
        Arg0: CodecInto<MultiValueEncoded<Env::Api, MultiValue2<node_storage::types::bls_key::BLSKey<Env::Api>, node_storage::types::bls_sig::BLSSignature<Env::Api>>>>,
    >(
        self,
        bls_keys_signatures: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("addNodes")
            .argument(&bls_keys_signatures)
            .original_result()
    }

    pub fn remove_nodes<
        Arg0: CodecInto<MultiValueEncoded<Env::Api, node_storage::types::bls_key::BLSKey<Env::Api>>>,
    >(
        self,
        bls_keys: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("removeNodes")
            .argument(&bls_keys)
            .original_result()
    }

    /// Each delegator gets a user id. This is in order to be able to iterate over their data. 
    /// This is a mapping from delegator address to delegator id. 
    /// The key is the bytes "user_id" concatenated with their public key. 
    /// The value is the user id. 
    pub fn get_user_id<
        Arg0: CodecInto<ManagedAddress<Env::Api>>,
    >(
        self,
        address: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, usize> {
        self.wrapped_tx
            .raw_call("getUserId")
            .argument(&address)
            .original_result()
    }

    pub fn get_user_address<
        Arg0: CodecInto<usize>,
    >(
        self,
        user_id: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ManagedAddress<Env::Api>> {
        self.wrapped_tx
            .raw_call("getUserAddress")
            .argument(&user_id)
            .original_result()
    }

    /// Retrieves the number of delegtors, including the owner, 
    /// even if they no longer have anything in the contract. 
    pub fn get_num_users(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, usize> {
        self.wrapped_tx
            .raw_call("getNumUsers")
            .original_result()
    }

    pub fn update_user_address<
        Arg0: CodecInto<MultiValueEncoded<Env::Api, ManagedAddress<Env::Api>>>,
    >(
        self,
        addresses: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, MultiValue3<usize, usize, usize>> {
        self.wrapped_tx
            .raw_call("updateUserAddress")
            .argument(&addresses)
            .original_result()
    }

    pub fn user_ids_without_address(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, MultiValueEncoded<Env::Api, usize>> {
        self.wrapped_tx
            .raw_call("userIdsWithoutAddress")
            .original_result()
    }

    pub fn fund_by_id<
        Arg0: CodecInto<usize>,
    >(
        self,
        id: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, user_fund_storage::types::fund_item::FundItem<Env::Api>> {
        self.wrapped_tx
            .raw_call("fundById")
            .argument(&id)
            .original_result()
    }

    pub fn get_total_stake(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("totalStake")
            .original_result()
    }

    /// Yields how much a user has staked in the contract. 
    pub fn get_user_total_stake_endpoint<
        Arg0: CodecInto<ManagedAddress<Env::Api>>,
    >(
        self,
        user_address: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getUserStake")
            .argument(&user_address)
            .original_result()
    }

    pub fn get_user_withdraw_only_stake<
        Arg0: CodecInto<ManagedAddress<Env::Api>>,
    >(
        self,
        user_address: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getUserWithdrawOnlyStake")
            .argument(&user_address)
            .original_result()
    }

    pub fn get_user_waiting_stake<
        Arg0: CodecInto<ManagedAddress<Env::Api>>,
    >(
        self,
        user_address: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getUserWaitingStake")
            .argument(&user_address)
            .original_result()
    }

    pub fn get_user_active_stake<
        Arg0: CodecInto<ManagedAddress<Env::Api>>,
    >(
        self,
        user_address: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getUserActiveStake")
            .argument(&user_address)
            .original_result()
    }

    pub fn get_user_unstaked_stake<
        Arg0: CodecInto<ManagedAddress<Env::Api>>,
    >(
        self,
        user_address: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getUserUnstakedStake")
            .argument(&user_address)
            .original_result()
    }

    pub fn get_user_deferred_payment_stake<
        Arg0: CodecInto<ManagedAddress<Env::Api>>,
    >(
        self,
        user_address: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getUserDeferredPaymentStake")
            .argument(&user_address)
            .original_result()
    }

    pub fn get_total_withdraw_only_stake(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getTotalWithdrawOnlyStake")
            .original_result()
    }

    pub fn get_total_waiting_stake(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getTotalWaitingStake")
            .original_result()
    }

    pub fn get_total_active_stake(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getTotalActiveStake")
            .original_result()
    }

    pub fn get_total_unstaked_stake(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getTotalUnstakedStake")
            .original_result()
    }

    pub fn get_total_deferred_payment_stake(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getTotalDeferredPaymentStake")
            .original_result()
    }

    pub fn get_user_stake_by_type_endpoint<
        Arg0: CodecInto<ManagedAddress<Env::Api>>,
    >(
        self,
        user_address: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, MultiValue5<BigUint<Env::Api>, BigUint<Env::Api>, BigUint<Env::Api>, BigUint<Env::Api>, BigUint<Env::Api>>> {
        self.wrapped_tx
            .raw_call("getUserStakeByType")
            .argument(&user_address)
            .original_result()
    }

    pub fn get_total_stake_by_type_endpoint(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, MultiValue5<BigUint<Env::Api>, BigUint<Env::Api>, BigUint<Env::Api>, BigUint<Env::Api>, BigUint<Env::Api>>> {
        self.wrapped_tx
            .raw_call("getTotalStakeByType")
            .original_result()
    }

    pub fn get_all_user_stake_by_type(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, MultiValueEncoded<Env::Api, MultiValue2<ManagedAddress<Env::Api>, MultiValue5<BigUint<Env::Api>, BigUint<Env::Api>, BigUint<Env::Api>, BigUint<Env::Api>, BigUint<Env::Api>>>>> {
        self.wrapped_tx
            .raw_call("getAllUserStakeByType")
            .original_result()
    }

    pub fn get_user_deferred_payment_list<
        Arg0: CodecInto<ManagedAddress<Env::Api>>,
    >(
        self,
        user_address: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, MultiValueEncoded<Env::Api, MultiValue2<BigUint<Env::Api>, u64>>> {
        self.wrapped_tx
            .raw_call("getUserDeferredPaymentList")
            .argument(&user_address)
            .original_result()
    }

    pub fn get_full_waiting_list(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, MultiValueEncoded<Env::Api, MultiValue3<ManagedAddress<Env::Api>, BigUint<Env::Api>, u64>>> {
        self.wrapped_tx
            .raw_call("getFullWaitingList")
            .original_result()
    }

    pub fn get_full_active_list(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, MultiValueEncoded<Env::Api, MultiValue2<ManagedAddress<Env::Api>, BigUint<Env::Api>>>> {
        self.wrapped_tx
            .raw_call("getFullActiveList")
            .original_result()
    }

    /// Owner activates specific nodes. 
    pub fn stake_nodes<
        Arg0: CodecInto<BigUint<Env::Api>>,
        Arg1: CodecInto<MultiValueManagedVec<Env::Api, node_storage::types::bls_key::BLSKey<Env::Api>>>,
    >(
        self,
        amount_to_stake: Arg0,
        bls_keys: Arg1,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("stakeNodes")
            .argument(&amount_to_stake)
            .argument(&bls_keys)
            .original_result()
    }

    /// Unstakes from the auction smart contract. 
    /// The nodes will stop receiving rewards, but stake cannot be yet reclaimed. 
    /// This operation is performed by the owner. 
    /// Does not unstake tokens. 
    pub fn unstake_nodes_endpoint<
        Arg0: CodecInto<MultiValueManagedVec<Env::Api, node_storage::types::bls_key::BLSKey<Env::Api>>>,
    >(
        self,
        bls_keys: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("unStakeNodes")
            .argument(&bls_keys)
            .original_result()
    }

    /// Unstakes from the auction smart contract. 
    /// The nodes will stop receiving rewards, but stake cannot be yet reclaimed. 
    /// This operation is performed by the owner. 
    /// Also unstakes tokens. 
    pub fn unstake_nodes_and_tokens_endpoint<
        Arg0: CodecInto<MultiValueManagedVec<Env::Api, node_storage::types::bls_key::BLSKey<Env::Api>>>,
    >(
        self,
        bls_keys: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("unStakeNodesAndTokens")
            .argument(&bls_keys)
            .original_result()
    }

    /// Owner can retry a callback in case of callback failure. 
    /// Warning: misuse can lead to state inconsistency. 
    pub fn force_node_unbond_period<
        Arg0: CodecInto<MultiValueManagedVec<Env::Api, node_storage::types::bls_key::BLSKey<Env::Api>>>,
    >(
        self,
        bls_keys: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("forceNodeUnBondPeriod")
            .argument(&bls_keys)
            .original_result()
    }

    /// Calls unbond for all provided nodes. Will fail if node cannot be unbonded. 
    pub fn unbond_specific_nodes_endpoint<
        Arg0: CodecInto<MultiValueManagedVec<Env::Api, node_storage::types::bls_key::BLSKey<Env::Api>>>,
    >(
        self,
        bls_keys: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("unBondNodes")
            .argument(&bls_keys)
            .original_result()
    }

    /// Calls unbond for all nodes that are in the unbond period and are due. 
    /// Nothing happens if no nodes can be unbonded. 
    pub fn unbond_all_possible_nodes(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("unBondAllPossibleNodes")
            .original_result()
    }

    /// Claims from auction SC funds that were sent but are not required to run the nodes. 
    pub fn claim_unused_funds(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("claimUnusedFunds")
            .original_result()
    }

    pub fn unjail_nodes<
        Arg0: CodecInto<MultiValueManagedVec<Env::Api, node_storage::types::bls_key::BLSKey<Env::Api>>>,
    >(
        self,
        bls_keys: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("unJailNodes")
            .argument(&bls_keys)
            .original_result()
    }

    pub fn unstake_tokens<
        Arg0: CodecInto<BigUint<Env::Api>>,
    >(
        self,
        amount: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("unStakeTokens")
            .argument(&amount)
            .original_result()
    }

    pub fn unbond_tokens<
        Arg0: CodecInto<BigUint<Env::Api>>,
    >(
        self,
        amount: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("unBondTokens")
            .argument(&amount)
            .original_result()
    }

    /// Yields the address of the contract with which staking will be performed. 
    /// This address is standard in the protocol, but it is saved in storage to avoid hardcoding it. 
    pub fn get_auction_contract_address(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, ManagedAddress<Env::Api>> {
        self.wrapped_tx
            .raw_call("getAuctionContractAddress")
            .original_result()
    }

    /// The proportion of rewards that goes to the owner as compensation for running the nodes. 
    /// 10000 = 100%. 
    pub fn get_service_fee(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getServiceFee")
            .original_result()
    }

    pub fn get_total_delegation_cap(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getTotalDelegationCap")
            .original_result()
    }

    pub fn is_bootstrap_mode(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, bool> {
        self.wrapped_tx
            .raw_call("isBootstrapMode")
            .original_result()
    }

    /// The minimum proportion of stake that has to be provided by the owner. 
    /// 10000 = 100%. 
    pub fn get_owner_min_stake_share(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getOwnerMinStakeShare")
            .original_result()
    }

    /// Minimum number of n_blocks between unstake and fund getting into inactive state. 
    pub fn get_n_blocks_before_unbond(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, u64> {
        self.wrapped_tx
            .raw_call("getNumBlocksBeforeUnBond")
            .original_result()
    }

    pub fn set_n_blocks_before_unbond_endpoint<
        Arg0: CodecInto<u64>,
    >(
        self,
        n_blocks_before_unbond: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("setNumBlocksBeforeUnBond")
            .argument(&n_blocks_before_unbond)
            .original_result()
    }

    /// Delegators are not allowed make transactions with less then this amount of stake (of any type). 
    /// Zero means disabled. 
    pub fn get_minimum_stake(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getMinimumStake")
            .original_result()
    }

    pub fn set_minimum_stake_endpoint<
        Arg0: CodecInto<BigUint<Env::Api>>,
    >(
        self,
        minimum_stake: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("setMinimumStake")
            .argument(&minimum_stake)
            .original_result()
    }

    pub fn global_op_checkpoint(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, GlobalOpCheckpoint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getGlobalOperationCheckpoint")
            .original_result()
    }

    pub fn is_global_op_in_progress(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, bool> {
        self.wrapped_tx
            .raw_call("isGlobalOperationInProgress")
            .original_result()
    }

    /// Yields all the rewards received by the contract since its creation. 
    /// This value is monotonously increasing - it can never decrease. 
    /// Handing out rewards will not decrease this value. 
    /// This is to keep track of how many funds entered the contract. It ignores any funds leaving the contract. 
    /// Individual rewards are computed based on this value. 
    /// For each user we keep a record on what was the value of the historical rewards when they last claimed. 
    /// Subtracting that from the current historical rewards yields how much accumulated in the contract since they last claimed. 
    pub fn get_total_cumulated_rewards(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getTotalCumulatedRewards")
            .original_result()
    }

    /// Yields how much a user is able to claim in rewards at the present time. 
    /// Does not update storage. 
    pub fn get_claimable_rewards<
        Arg0: CodecInto<ManagedAddress<Env::Api>>,
    >(
        self,
        user: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getClaimableRewards")
            .argument(&user)
            .original_result()
    }

    /// Utility readonly function to check how many unclaimed rewards currently reside in the contract. 
    pub fn get_total_unclaimed_rewards(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getTotalUnclaimedRewards")
            .original_result()
    }

    pub fn total_unprotected(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getTotalUnProtected")
            .original_result()
    }

    /// Invariant: should never return error. 
    pub fn validate_owner_stake_share(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("validateOwnerStakeShare")
            .original_result()
    }

    /// Invariant: should never return error. 
    pub fn validate_delegation_cap_invariant(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("validateDelegationCapInvariant")
            .original_result()
    }

    /// Continues executing any interrupted operation. 
    /// Returns true if still out of gas, false if computation completed. 
    pub fn continue_global_operation_endpoint(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, multiversx_sc::types::io::operation_completion_status::OperationCompletionStatus> {
        self.wrapped_tx
            .raw_call("continueGlobalOperation")
            .original_result()
    }

    /// Total delegation cap can be modified by owner only. 
    /// It will recalculate and set the checkpoint for all the delegators 
    pub fn modify_total_delegation_cap<
        Arg0: CodecInto<BigUint<Env::Api>>,
    >(
        self,
        new_total_cap: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, multiversx_sc::types::io::operation_completion_status::OperationCompletionStatus> {
        self.wrapped_tx
            .raw_call("modifyTotalDelegationCap")
            .argument(&new_total_cap)
            .original_result()
    }

    /// The stake per node can be changed by the owner. 
    /// It does not get set in the contructor, so the owner has to manually set it after the contract is deployed. 
    pub fn set_service_fee_endpoint<
        Arg0: CodecInto<usize>,
    >(
        self,
        service_fee_per_10000: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, multiversx_sc::types::io::operation_completion_status::OperationCompletionStatus> {
        self.wrapped_tx
            .raw_call("setServiceFee")
            .argument(&service_fee_per_10000)
            .original_result()
    }

    /// Retrieve those rewards to which the caller is entitled. 
    /// Will send: 
    /// - new rewards 
    /// - rewards that were previously computed but not sent 
    pub fn claim_rewards(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("claimRewards")
            .original_result()
    }

    /// Delegate stake to the smart contract. 
    /// Stake is initially inactive, so does it not produce rewards. 
    pub fn stake_endpoint(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("stake")
            .original_result()
    }

    /// unStake - the user will announce that he wants to get out of the contract 
    /// selected funds will change from active to inactive, but claimable only after unBond period ends 
    pub fn unstake_endpoint<
        Arg0: CodecInto<BigUint<Env::Api>>,
    >(
        self,
        amount: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("unStake")
            .argument(&amount)
            .original_result()
    }

    pub fn get_unstakeable<
        Arg0: CodecInto<ManagedAddress<Env::Api>>,
    >(
        self,
        user_address: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getUnStakeable")
            .argument(&user_address)
            .original_result()
    }

    pub fn unbond_user(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("unBond")
            .original_result()
    }

    pub fn get_unbondable<
        Arg0: CodecInto<ManagedAddress<Env::Api>>,
    >(
        self,
        user_address: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .raw_call("getUnBondable")
            .argument(&user_address)
            .original_result()
    }

    /// Raw id of the last checkpoint reached by any of the dust cleanup endpoints. 
    pub fn dust_cleanup_checkpoint(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, usize> {
        self.wrapped_tx
            .raw_call("dustCleanupCheckpoint")
            .original_result()
    }

    /// Counts fund buckets in the waiting list that are below a certain threshold. 
    /// Unlike most views, yields the number of entries, rather than the sum of EGLD. 
    pub fn count_dust_items_waiting_list<
        Arg0: CodecInto<BigUint<Env::Api>>,
    >(
        self,
        dust_limit: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, usize> {
        self.wrapped_tx
            .raw_call("countDustItemsWaitingList")
            .argument(&dust_limit)
            .original_result()
    }

    /// Counts fund buckets in the active staking list that are below a certain threshold. 
    /// Unlike most views, yields the number of entries, rather than the sum of EGLD. 
    pub fn count_dust_items_active<
        Arg0: CodecInto<BigUint<Env::Api>>,
    >(
        self,
        dust_limit: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, usize> {
        self.wrapped_tx
            .raw_call("countDustItemsActive")
            .argument(&dust_limit)
            .original_result()
    }

    /// Unstakes all fund buckets in the waiting list that are below a certin threshold. 
    /// Will stop if running low on gas. 
    /// Does not block the rest of the contract. If any operation interferes with an interrupted 
    /// dust cleanup, the operation can be begun again. 
    /// It will auto-reset if the list ends or the current item is no longer valid. 
    pub fn dust_cleanup_waiting_list<
        Arg0: CodecInto<BigUint<Env::Api>>,
    >(
        self,
        dust_limit: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("dustCleanupWaitingList")
            .argument(&dust_limit)
            .original_result()
    }

    /// Unstakes and unbonds all active fund buckets that are below a certin threshold. 
    /// Unlike the regular unstake/unbond process, it will send the funds directly in `WithdrawOnly` state. 
    /// Will stop if running low on gas. 
    /// Does not block the rest of the contract. If any operation interferes with an interrupted 
    /// dust cleanup, the operation can be begun again. 
    /// It will auto-reset if the list ends or the current item is no longer valid. 
    pub fn dust_cleanup_active<
        Arg0: CodecInto<BigUint<Env::Api>>,
    >(
        self,
        dust_limit: Arg0,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("dustCleanupActive")
            .argument(&dust_limit)
            .original_result()
    }

    pub fn dns_register<
        Arg0: CodecInto<ManagedAddress<Env::Api>>,
        Arg1: CodecInto<ManagedBuffer<Env::Api>>,
    >(
        self,
        dns_address: Arg0,
        name: Arg1,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("dnsRegister")
            .argument(&dns_address)
            .argument(&name)
            .original_result()
    }

    pub fn set_feature_flag_endpoint<
        Arg0: CodecInto<ManagedBuffer<Env::Api>>,
        Arg1: CodecInto<bool>,
    >(
        self,
        feature_name: Arg0,
        value: Arg1,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("setFeatureFlag")
            .argument(&feature_name)
            .argument(&value)
            .original_result()
    }

    pub fn pause_endpoint(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("pause")
            .original_result()
    }

    pub fn unpause_endpoint(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, ()> {
        self.wrapped_tx
            .raw_call("unpause")
            .original_result()
    }

    pub fn paused_status(
        self,
    ) -> TxProxyCall<Env, From, To, Gas, bool> {
        self.wrapped_tx
            .raw_call("isPaused")
            .original_result()
    }
}

#[rustfmt::skip]
#[derive(TopEncode, TopDecode)]
pub enum GlobalOpCheckpoint<Api>
where
    Api: ManagedTypeApi,
{
    None,
    ModifyTotalDelegationCap(ModifyTotalDelegationCapData<Api>),
    ChangeServiceFee {
        new_service_fee: BigUint<Api>,
        compute_rewards_data: ComputeAllRewardsData<Api>,
    },
}

#[derive(TopEncode, TopDecode)]
pub struct ModifyTotalDelegationCapData<Api>
where
    Api: ManagedTypeApi,
{
    pub new_delegation_cap: BigUint<Api>,
    pub remaining_swap_waiting_to_active: BigUint<Api>,
    pub remaining_swap_active_to_def_p: BigUint<Api>,
    pub remaining_swap_unstaked_to_def_p: BigUint<Api>,
    pub step: ModifyDelegationCapStep<Api>,
}

#[derive(TopEncode, TopDecode)]
pub enum ModifyDelegationCapStep<Api>
where
    Api: ManagedTypeApi,
{
    ComputeAllRewards(ComputeAllRewardsData<Api>),
    SwapWaitingToActive,
    SwapUnstakedToDeferredPayment,
    SwapActiveToDeferredPayment,
}

#[derive(TopEncode, TopDecode)]
pub struct ComputeAllRewardsData<Api>
where
    Api: ManagedTypeApi,
{
    pub last_id: usize,
    pub sum_unclaimed: BigUint<Api>,
    pub rewards_checkpoint: BigUint<Api>,
}
