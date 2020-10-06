
#![no_std]
#![allow(unused_attributes)]
#![allow(clippy::string_lit_as_bytes)]

use delegation_v0_5::*;

imports!();

#[elrond_wasm_derive::contract(DelegationImpl)]
pub trait Delegation {

    // METADATA

    #[endpoint]
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    // MODULES

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(NodeConfigModuleImpl)]
    fn node_config(&self) -> NodeConfigModuleImpl<T, BigInt, BigUint>;

    #[module(RewardsModuleImpl)]
    fn rewards(&self) -> RewardsModuleImpl<T, BigInt, BigUint>;

    #[module(ResetCheckpointsModuleImpl)]
    fn reset_checkpoints(&self) -> ResetCheckpointsModuleImpl<T, BigInt, BigUint>;

    #[module(SettingsModuleImpl)]
    fn settings(&self) -> SettingsModuleImpl<T, BigInt, BigUint>;

    #[module(PauseModuleImpl)]
    fn pause(&self) -> PauseModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn user_stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;

    #[module(NodeActivationModuleImpl)]
    fn node_activation(&self) -> NodeActivationModuleImpl<T, BigInt, BigUint>;

    #[module(UserUnStakeModuleImpl)]
    fn user_unstake(&self) -> UserUnStakeModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(FundViewModuleImpl)]
    fn fund_view_module(&self) -> FundViewModuleImpl<T, BigInt, BigUint>;

    // INIT

    /// This is the contract constructor, called only once when the contract is deployed.
    #[init]
    fn init(&self,
        auction_contract_addr: &Address,
        service_fee_per_10000: usize,
        owner_min_stake_share_per_10000: usize,
        n_blocks_before_unbond: u64,
        minimum_stake: BigUint,
        total_delegation_cap: BigUint,
    ) -> SCResult<()> {

        let owner = self.get_caller();
        self.user_data().set_user_id(&owner, OWNER_USER_ID.get()); // node reward destination will be user #1
        self.user_data().set_user_address(OWNER_USER_ID.get(), &owner);
        self.user_data().set_num_users(1);

        self.settings().set_auction_addr(&auction_contract_addr);

        require!(service_fee_per_10000 <= PERCENTAGE_DENOMINATOR,
            "service fee out of range");

        let next_service_fee = BigUint::from(service_fee_per_10000);
        self.settings().set_service_fee(next_service_fee);

        sc_try!(self.settings().set_owner_min_stake_share_validated(owner_min_stake_share_per_10000));

        self.settings().set_n_blocks_before_unbond(n_blocks_before_unbond);
        self.settings().set_minimum_stake(&minimum_stake);

        self.settings().set_total_delegation_cap(total_delegation_cap);
        self.settings().set_bootstrap_mode(true);

        Ok(())
    }

    // Callbacks can only be declared here for the moment.

    #[callback]
    fn auction_stake_callback(&self,
            #[callback_arg] node_ids: Vec<usize>,
            call_result: AsyncCallResult<VarArgs<BLSStatusMultiArg>>) {

        self.node_activation().auction_stake_callback(
            node_ids,
            call_result).unwrap();
        // TODO: replace unwrap with typical Result handling
    }

    #[callback]
    fn auction_unstake_callback(&self,
            #[callback_arg] node_ids: Vec<usize>,
            call_result: AsyncCallResult<VarArgs<BLSStatusMultiArg>>) {

        self.node_activation().auction_unstake_callback(
            node_ids,
            call_result).unwrap();
            // TODO: replace unwrap with typical Result handling
    }

    #[callback]
    fn auction_unbond_callback(&self,
            #[callback_arg] node_ids: Vec<usize>,
            call_result: AsyncCallResult<VarArgs<BLSStatusMultiArg>>) {

        self.node_activation().auction_unbond_callback(
            node_ids,
            call_result).unwrap();
            // TODO: replace unwrap with typical Result handling
    }

}
