
use crate::user_data::*;

/// Indicates how we express the percentage of rewards that go to the node.
/// Since we cannot have floating point numbers, we use fixed point with this denominator.
/// Percents + 2 decimals -> 10000.
pub static SERVICE_FEE_DENOMINATOR: u64 = 10000;

/// Validator reward destination will always be user with id 1.
/// This can also count as a delegator (if the owner adds stake into the contract) or not.
pub static OWNER_USER_ID: usize = 1;

imports!();

/// The module deals with initializaton and the global contract settings.
/// 
#[elrond_wasm_derive::module(SettingsModuleImpl)]
pub trait SettingsModule {

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    /// This is the contract constructor, called only once when the contract is deployed.
    /// 
    fn init(&self,
        service_fee_per_10000: BigUint,
        auction_contract_addr: &Address,
        time_before_force_unstake: u64,
    ) -> Result<(), &str> {

        if service_fee_per_10000 > SERVICE_FEE_DENOMINATOR {
            return Err("node share out of range");
        }
        self._set_service_fee(&service_fee_per_10000);

        let owner = self.get_caller();
        self._set_owner(&owner);

        self._set_node_reward_destination(&owner);
        self.user_data()._set_user_id(&owner, OWNER_USER_ID); // node reward destination will be user #1
        self.user_data()._set_num_users(1);

        self._set_auction_addr(&auction_contract_addr);

        self._set_time_before_force_unstake(time_before_force_unstake);

        Ok(())
    }

    /// Yields the address of the contract with which staking will be performed.
    #[view]
    #[storage_get("owner")]
    fn getContractOwner(&self) -> Address;

    #[private]
    #[storage_set("owner")]
    fn _set_owner(&self, owner: &Address);

    /// This is the address where the contract owner receives the rewards for running the nodes.
    /// It can in principle be different from the owner address.
    #[view]
    #[storage_get("node_rewards_addr")]
    fn getNodeRewardDestination(&self) -> Address;

    #[private]
    #[storage_set("node_rewards_addr")]
    fn _set_node_reward_destination(&self, nrd: &Address);

    /// Yields the address of the contract with which staking will be performed.
    /// This address is standard in the protocol, but it is saved in storage to avoid hardcoding it.
    #[view]
    #[storage_get("auction_addr")]
    fn getAuctionContractAddress(&self) -> Address;

    #[private]
    #[storage_set("auction_addr")]
    fn _set_auction_addr(&self, auction_addr: &Address);


    /// Delegators can force the entire contract to unstake
    /// if they put up stake for sale and no-one is buying it.
    /// However, they need to wait this much time (in milliseconds)
    /// from when the put up the stake for sale and the moment they can force global unstaking.
    #[view]
    #[storage_get("time_before_force_unstake")]
    fn getTimeBeforeForceUnstake(&self) -> u64;

    #[private]
    #[storage_set("time_before_force_unstake")]
    fn _set_time_before_force_unstake(&self, time_before_force_unstake: u64);

    /// The proportion of rewards that goes to the owner as compensation for running the nodes.
    /// 10000 = 100%.
    #[view]
    #[storage_get("service_fee")]
    fn getServiceFee(&self) -> BigUint;

    #[private]
    #[storage_set("service_fee")]
    fn _set_service_fee(&self, service_fee: &BigUint);
}
