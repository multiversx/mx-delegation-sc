
use user_fund_storage::user_data::*;
use user_fund_storage::fund_transf_module::*;
use node_storage::node_config::*;
use crate::rewards::*;
use crate::reset_checkpoints::*;
use crate::extended_comp_types::*;

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

    #[module(ResetCheckpointsModuleImpl)]
    fn reset_checkpoints(&self) -> ResetCheckpointsModuleImpl<T, BigInt, BigUint>;

    /// This is the contract constructor, called only once when the contract is deployed.
    #[init]
    fn init(&self,
        auction_contract_addr: &Address,
        service_fee_per_10000: usize,
        owner_min_stake_share_per_10000: usize,
        n_blocks_before_unbond: u64,
        minimum_stake: BigUint,
    ) -> SCResult<()> {

        let owner = self.get_caller();
        self.user_data().set_user_id(&owner, OWNER_USER_ID); // node reward destination will be user #1
        self.user_data().set_num_users(1);

        self.set_auction_addr(&auction_contract_addr);

        if service_fee_per_10000 > PERCENTAGE_DENOMINATOR {
            return sc_error!("service fee out of range");
        }
        let next_service_fee = BigUint::from(service_fee_per_10000);
        self.set_service_fee(next_service_fee);

        sc_try!(self.set_owner_min_stake_share_validated(owner_min_stake_share_per_10000));

        self.set_n_blocks_before_unbond(n_blocks_before_unbond);
        let min_stake_2 = minimum_stake.clone();
        self.set_minimum_stake(minimum_stake);
        self.set_total_delegation_cap(min_stake_2);

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
    fn set_service_fee(&self, service_fee: BigUint);

    /// The stake per node can be changed by the owner.
    /// It does not get set in the contructor, so the owner has to manually set it after the contract is deployed.
    #[endpoint(setServiceFee)]
    fn set_service_fee_endpoint(&self, service_fee_per_10000: usize) -> SCResult<bool> {
        require!(self.owner_called(),
            "only owner can change service fee");

        require!(service_fee_per_10000 <= PERCENTAGE_DENOMINATOR,
            "service fee out of range");

        require!(!self.reset_checkpoints().is_interrupted_computation(),
            "global checkpoint is in progress");
        
        let new_service_fee = BigUint::from(service_fee_per_10000);
        if self.get_service_fee() == new_service_fee {
            return Ok(COMPUTATION_DONE)
        }

        self.reset_checkpoints().perform_extended_computation(ExtendedComputation::ChangeServiceFee{
            new_service_fee,
            compute_rewards_data: ComputeAllRewardsData::new(self.get_block_epoch()),
        })
    }
    
    #[view(getTotalDelegationCap)]
    #[storage_get("total_delegation_cap")]
    fn get_total_delegation_cap(&self) -> BigUint;

    #[storage_set("total_delegation_cap")]
    fn set_total_delegation_cap(&self, amount: BigUint);

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

        self.set_owner_min_stake_share(owner_min_stake_share_per_10000);
        Ok(())
    }

    /// Minimum number of n_blocks between unstake and fund getting into inactive state.
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

    /// Delegators are not allowed make transactions with less then this amount of stake (of any type).
    /// Zero means disabled.
    #[view(getMinimumStake)]
    #[storage_get("min_stake")]
    fn get_minimum_stake(&self) -> BigUint;

    #[storage_set("min_stake")]
    fn set_minimum_stake(&self, minimum_stake: BigUint);

    #[endpoint(setMinimumStake)]
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
