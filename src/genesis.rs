imports!();

use crate::node_state::*;
use crate::user_stake_state::*;

use crate::events::*;
use crate::node_activation::*;
use crate::node_config::*;
use crate::user_data::*;
use crate::fund_transf_module::*;
use crate::fund_view_module::*;
use crate::user_stake::*;

#[elrond_wasm_derive::module(GenesisModuleImpl)]
pub trait GenesisModule {

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn user_stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(FundTransformationsModuleImpl)]
    fn fund_transf_module(&self) -> FundTransformationsModuleImpl<T, BigInt, BigUint>;

    #[module(FundViewModuleImpl)]
    fn fund_view_module(&self) -> FundViewModuleImpl<T, BigInt, BigUint>;

    #[module(NodeConfigModuleImpl)]
    fn node_config(&self) -> NodeConfigModuleImpl<T, BigInt, BigUint>;

    #[module(NodeActivationModuleImpl)]
    fn node_activation(&self) -> NodeActivationModuleImpl<T, BigInt, BigUint>;


    /// Function to be used only during genesis block.
    /// Cannot perform payments during genesis block, so we update state but not the balance.
    #[endpoint(stakeGenesis)]
    fn stake_genesis(&self, stake: BigUint) -> SCResult<()> {
        if self.get_block_nonce() > 0 {
            return sc_error!("genesis block only")
        }
        self.user_stake().process_stake(stake)
    }

    /// Function to be used only once, during genesis block.
    /// Cannot perform payments during genesis block, so we update state but do not receive or send funds.
    #[endpoint(activateGenesis)]
    fn activate_genesis(&self) -> SCResult<()> {
        if self.get_block_nonce() > 0 {
            return sc_error!("genesis block only")
        }

        // set nodes to Active, and count how many not deleted
        let num_nodes = self.node_config().get_num_nodes();
        let mut num_inactive_nodes = 0usize;
        for node_id in 1..num_nodes+1 {
            match self.node_config().get_node_state(node_id) {
                NodeState::Inactive => {
                    self.node_config().set_node_state(node_id, NodeState::Active);
                    num_inactive_nodes += 1;
                },
                NodeState::Removed => {},
                _ => {
                    return sc_error!("cannot activate twice during genesis");
                },
            }
        }

        // validate that node stake and user stake match
        let stake_required_by_nodes = BigUint::from(num_inactive_nodes) * self.node_config().get_stake_per_node();
        let mut total_inactive_stake = self.fund_view_module().get_user_stake_of_type(USER_STAKE_TOTALS_ID, UserStakeState::Inactive);
        if stake_required_by_nodes != total_inactive_stake {
            return sc_error!("stake required by nodes must match total user stake at genesis");
        }

        // convert all user inactive stake to active stake
        // self.user_data().convert_user_stake_asc(UserStakeState::Inactive, UserStakeState::Active, &mut total_inactive_stake);
        // if total_inactive_stake > 0 {
        //     return sc_error!("not enough active stake");
        // }
        let mut total_inactive_stake_2 = total_inactive_stake.clone();
        sc_try!(self.fund_transf_module().activate_start_transf(&mut total_inactive_stake));
        sc_try!(self.fund_transf_module().activate_finish_ok_transf(&mut total_inactive_stake_2));

        // log event (no data)
        self.events().activation_ok_event(());

        Ok(())
    }

}