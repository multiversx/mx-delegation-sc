
use user_fund_storage::user_data::*;
use user_fund_storage::fund_transf_module::*;
use node_storage::node_config::*;
use crate::rewards::*;

/// Indicates how we express the percentage of rewards that go to the node.
/// Since we cannot have floating point numbers, we use fixed point with this denominator.
/// Percents + 2 decimals -> 10000.
pub static PERCENTAGE_DENOMINATOR: usize = 10000;

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

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(NodeConfigModuleImpl)]
    fn node_config(&self) -> NodeConfigModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    /// This is the contract constructor, called only once when the contract is deployed.
    #[init]
    fn init(&self, service_fee_per_10000: usize) -> SCResult<()> {
        sc_try!(self.set_service_fee_validated(service_fee_per_10000));

        Ok(())
    }

    fn owner_called(&self) -> bool {
        self.get_caller() == self.get_owner_address()
    }

    /// Yields the address of the contract with which staking will be performed.
    /// This address is standard in the protocol, but it is saved in storage to avoid hardcoding it.
    #[view(getAuctionContractAddress)]
    #[storage_get("auction_addr")]
    fn get_auction_contract_address(&self) -> Address;

    #[storage_set("auction_addr")]
    fn set_auction_addr(&self, auction_addr: &Address);


    /// The proportion of rewards that goes to the owner as compensation for running the nodes.
    /// 10000 = 100%.
    #[view(getServiceFee)]
    #[storage_get("service_fee")]
    fn get_service_fee(&self) -> BigUint;

    #[storage_set("service_fee")]
    fn set_service_fee(&self, service_fee: usize);

    fn set_service_fee_validated(&self, service_fee_per_10000: usize) -> SCResult<()> {
        if service_fee_per_10000 > PERCENTAGE_DENOMINATOR {
            return sc_error!("service fee out of range");
        }

        self.set_service_fee(service_fee_per_10000);
        Ok(())
    }

    /// The service fee can be changed by the owner.
    #[endpoint(setServiceFee)]
    fn set_service_fee_endpoint(&self, service_fee_per_10000: usize) -> SCResult<()> {
        if !self.owner_called() {
            return sc_error!("only owner can change service fee"); 
        }

        sc_try!(self.rewards().compute_all_rewards());

        self.set_service_fee_validated(service_fee_per_10000)
    }
    
    /// How much stake has to be provided per validator node.
    /// After genesis this sum is fixed to 2,500,000 ERD, but at some point bidding will happen.
    #[view(getStakePerNode)]
    #[storage_get("stake_per_node")]
    fn get_stake_per_node(&self) -> BigUint;

    #[storage_set("stake_per_node")]
    fn set_stake_per_node(&self, spn: &BigUint);

    /// The stake per node can be changed by the owner.
    /// It does not get set in the contructor, so the owner has to manually set it after the contract is deployed.
    #[endpoint(setStakePerNode)]
    fn set_stake_per_node_endpoint(&self, stake_per_node: &BigUint) -> SCResult<()> {
        if !self.owner_called() {
            return sc_error!("only owner can change stake per node"); 
        }

        // check that all nodes idle
        if !self.node_config().all_nodes_idle() {
            return sc_error!("cannot change stake per node while at least one node is active");
        }

        self.set_stake_per_node(&stake_per_node);
        Ok(())
    }

    /// The minimum proportion of stake that has to be provided by the owner.
    /// 10000 = 100%.
    #[view(getOwnerMinStakeShare)]
    #[storage_get("owner_min_stake_share")]
    fn get_owner_min_stake_share(&self) -> BigUint;

    #[storage_set("owner_min_stake_share")]
    fn set_owner_min_stake_share(&self, owner_min_stake_share: usize);

    fn set_owner_min_stake_share_validated(&self, owner_min_stake_share_per_10000: usize) -> SCResult<()> {
        if owner_min_stake_share_per_10000 > PERCENTAGE_DENOMINATOR {
            return sc_error!("owner min stake share out of range");
        }

        self.set_service_fee(owner_min_stake_share_per_10000);
        Ok(())
    }

    /// Delegators can force the entire contract to unstake
    /// if they put up stake for sale and no-one is buying it.
    /// However, they need to wait for this many n_blocks to be processed in between,
    /// from when the put up the stake for sale and the moment they can force global unstaking.
    #[view(getNumBlocksBeforeForceUnstake)]
    #[storage_get("n_blocks_before_force_unstake")]
    fn get_n_blocks_before_force_unstake(&self) -> u64;

    #[storage_set("n_blocks_before_force_unstake")]
    fn set_n_blocks_before_force_unstake(&self, n_blocks_before_force_unstake: u64);

    /// Minimum number of n_blocks between unstake and unbond.
    /// This mirrors the minimum unbonding period in the auction SC. 
    /// The auction SC cannot be cheated by setting this field lower, unbond will fail anyway if attempted too early.
    #[view(getNumBlocksBeforeUnBond)]
    #[storage_get("n_blocks_before_unbond")]
    fn get_n_blocks_before_unbond(&self) -> u64;

    #[storage_set("n_blocks_before_unbond")]
    fn set_n_blocks_before_unbond(&self, n_blocks_before_unbond: u64);

    /// Determines who can call stakeAllAvailable.
    /// If true, anyone can call.
    /// If false, only owner can call.
    #[view(anyoneCanActivate)]
    #[storage_get("anyone_can_activate")]
    fn anyone_can_activate(&self) -> bool;

    #[storage_set("anyone_can_activate")]
    fn set_anyone_can_activate(&self, anyone_can_activate: bool);

    #[endpoint(setAnyoneCanActivate)]
    fn set_anyone_can_activate_endpoint(&self) -> SCResult<()>{
        if !self.owner_called() {
            return sc_error!("only owner can enable auto activation");
        }
        self.set_anyone_can_activate(true);
        Ok(())
    }

    #[endpoint(setOnlyOwnerCanActivate)]
    fn set_only_owner_can_activate_endpoint(&self) -> SCResult<()>{
        if !self.owner_called() {
            return sc_error!("only owner can disable auto activation");
        }
        self.set_anyone_can_activate(false);
        Ok(())
    }

    fn caller_can_activate(&self) -> bool {
        self.anyone_can_activate() || self.owner_called()
    }

    /// Delegators are not allowed to hold more than zero but less than this amount of stake (of any type).
    /// Zero means disabled.
    #[view(getMinimumStake)]
    #[storage_get("min_stake")]
    fn get_minimum_stake(&self) -> BigUint;

    #[storage_set("min_stake")]
    fn set_minimum_stake(&self, minimum_stake: BigUint);

    #[view(setMinimumStake)]
    fn set_minimum_stake_endpoint(&self, minimum_stake: BigUint) -> SCResult<()> {
        if !self.owner_called() {
            return sc_error!("only owner can set minimum stake");
        }
        self.set_minimum_stake(minimum_stake);
        Ok(())
    }

    /// The ability to disable unstaking is not normally part of a delegation smart contract.
    /// It gives a malitious owner the ability to block all delegator stake in the contract indefinitely. 
    /// However, it will be used by Elrond in the period immediately after genesis.
    /// In this version of the contract unstaking is disabled by default and needs to be enabled by the owner explicitly.
    #[view(unStakeEnabled)]
    #[storage_get("unstake_enabled")]
    fn is_unstake_enabled(&self) -> bool;

    #[storage_set("unstake_enabled")]
    fn set_unstake_enabled(&self, unstake_enabled: bool);

    #[endpoint(enableUnStake)]
    fn enable_unstake(&self) -> SCResult<()>{
        if !self.owner_called() {
            return sc_error!("only owner can enable unStake");
        }
        self.set_unstake_enabled(true);
        Ok(())
    }

    #[endpoint(disableUnStake)]
    fn disable_unstake(&self) -> SCResult<()>{
        if !self.owner_called() {
            return sc_error!("only owner can disable unStake");
        }
        self.set_unstake_enabled(false);
        Ok(())
    }
}
