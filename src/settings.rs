
use crate::user_data::*;

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
        auction_contract_addr: &Address,
        n_blocks_before_force_unstake: u64,
        n_blocks_before_unbond: u64,
    ) -> Result<(), &str> {

        let owner = self.get_caller();
        self._set_owner(&owner);

        self._set_node_reward_destination(&owner);
        self.user_data()._set_user_id(&owner, OWNER_USER_ID); // node reward destination will be user #1
        self.user_data()._set_num_users(1);

        self._set_auction_addr(&auction_contract_addr);

        self._set_n_blocks_before_force_unstake(n_blocks_before_force_unstake);
        self._set_n_blocks_before_unbond(n_blocks_before_unbond);

        Ok(())
    }

    /// Yields the address of the contract with which staking will be performed.
    #[view]
    #[storage_get("owner")]
    fn getContractOwner(&self) -> Address;

    #[private]
    #[storage_set("owner")]
    fn _set_owner(&self, owner: &Address);

    #[private]
    fn _owner_called(&self) -> bool {
        self.get_caller() == self.getContractOwner()
    }

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
    /// However, they need to wait for this many n_blocks to be processed in between,
    /// from when the put up the stake for sale and the moment they can force global unstaking.
    #[view]
    #[storage_get("n_blocks_before_force_unstake")]
    fn getNumBlocksBeforeForceUnstake(&self) -> u64;

    #[private]
    #[storage_set("n_blocks_before_force_unstake")]
    fn _set_n_blocks_before_force_unstake(&self, n_blocks_before_force_unstake: u64);

    /// Minimum number of n_blocks between unstake and unbond.
    /// This mirrors the minimum unbonding period in the auction SC. 
    /// The auction SC cannot be cheated by setting this field lower, unbond will fail anyway if attempted too early.
    #[view]
    #[storage_get("n_blocks_before_unbond")]
    fn getNumBlocksBeforeUnBond(&self) -> u64;

    #[private]
    #[storage_set("n_blocks_before_unbond")]
    fn _set_n_blocks_before_unbond(&self, n_blocks_before_unbond: u64);

    #[view]
    #[storage_get("auto_activation_enabled")]
    fn isAutoActivationEnabled(&self) -> bool;

    #[private]
    #[storage_set("auto_activation_enabled")]
    fn _set_auto_activation_enabled(&self, auto_activation_enabled: bool);

    fn enableAutoActivation(&self) -> Result<(), &str>{
        if self.get_caller() != self.getContractOwner() {
            return Err("only owner can enable auto activation");
        }
        self._set_auto_activation_enabled(true);
        Ok(())
    }

    fn disableAutoActivation(&self) -> Result<(), &str>{
        if self.get_caller() != self.getContractOwner() {
            return Err("only owner can disable auto activation");
        }
        self._set_auto_activation_enabled(false);
        Ok(())
    }

}
