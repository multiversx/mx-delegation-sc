use core::num::NonZeroUsize;

multiversx_sc::imports!();

/// Indicates how we express the percentage of rewards that go to the node.
/// Since we cannot have floating point numbers, we use fixed point with this denominator.
/// Percents + 2 decimals -> 10000.
pub const PERCENTAGE_DENOMINATOR: usize = 10000;

/// Validator reward destination will always be user with id 1.
/// This can also count as a delegator (if the owner adds stake into the contract) or not.
pub const OWNER_USER_ID: NonZeroUsize = NonZeroUsize::new(1).unwrap();

/// The module deals with initializaton and the global contract settings.
///
#[multiversx_sc::derive::module]
pub trait SettingsModule {
    /// Yields the address of the contract with which staking will be performed.
    /// This address is standard in the protocol, but it is saved in storage to avoid hardcoding it.
    #[view(getAuctionContractAddress)]
    #[storage_get("auction_addr")]
    fn get_auction_contract_address(&self) -> ManagedAddress;

    #[storage_set("auction_addr")]
    fn set_auction_addr(&self, auction_addr: &ManagedAddress);

    /// The proportion of rewards that goes to the owner as compensation for running the nodes.
    /// 10000 = 100%.
    #[view(getServiceFee)]
    #[storage_get("service_fee")]
    fn get_service_fee(&self) -> BigUint;

    #[storage_set("service_fee")]
    fn set_service_fee(&self, service_fee: BigUint);

    #[view(getTotalDelegationCap)]
    #[storage_get("total_delegation_cap")]
    fn get_total_delegation_cap(&self) -> BigUint;

    #[storage_set("total_delegation_cap")]
    fn set_total_delegation_cap(&self, amount: BigUint);

    #[view(isBootstrapMode)]
    #[storage_get("bootstrap_mode")]
    fn is_bootstrap_mode(&self) -> bool;

    #[storage_set("bootstrap_mode")]
    fn set_bootstrap_mode(&self, bootstrap_mode: bool);

    /// The minimum proportion of stake that has to be provided by the owner.
    /// 10000 = 100%.
    #[view(getOwnerMinStakeShare)]
    #[storage_get("owner_min_stake_share")]
    fn get_owner_min_stake_share(&self) -> BigUint;

    #[storage_set("owner_min_stake_share")]
    fn set_owner_min_stake_share(&self, owner_min_stake_share: usize);

    fn set_owner_min_stake_share_validated(&self, owner_min_stake_share_per_10000: usize) {
        require!(
            owner_min_stake_share_per_10000 <= PERCENTAGE_DENOMINATOR,
            "owner min stake share out of range"
        );

        self.set_owner_min_stake_share(owner_min_stake_share_per_10000);
    }

    /// Minimum number of n_blocks between unstake and fund getting into inactive state.
    #[view(getNumBlocksBeforeUnBond)]
    #[storage_get("n_blocks_before_unbond")]
    fn get_n_blocks_before_unbond(&self) -> u64;

    #[storage_set("n_blocks_before_unbond")]
    fn set_n_blocks_before_unbond(&self, n_blocks_before_unbond: u64);

    #[only_owner]
    #[endpoint(setNumBlocksBeforeUnBond)]
    fn set_n_blocks_before_unbond_endpoint(&self, n_blocks_before_unbond: u64) {
        self.set_n_blocks_before_unbond(n_blocks_before_unbond);
    }

    /// Delegators are not allowed make transactions with less then this amount of stake (of any type).
    /// Zero means disabled.
    #[view(getMinimumStake)]
    #[storage_get("min_stake")]
    fn get_minimum_stake(&self) -> BigUint;

    #[storage_set("min_stake")]
    fn set_minimum_stake(&self, minimum_stake: &BigUint);

    #[only_owner]
    #[endpoint(setMinimumStake)]
    fn set_minimum_stake_endpoint(&self, minimum_stake: BigUint) {
        self.set_minimum_stake(&minimum_stake);
    }
}
