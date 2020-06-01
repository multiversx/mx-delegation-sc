imports!();

use crate::node_state::*;
use crate::user_stake_state::*;

use crate::events::*;
use crate::node_activation::*;
use crate::node_config::*;
use crate::user_data::*;
use crate::user_stake::*;

#[elrond_wasm_derive::module(GenesisModuleImpl)]
pub trait GenesisModule {

    #[module(EventsModuleImpl)]
    fn events(&self) -> EventsModuleImpl<T, BigInt, BigUint>;

    #[module(UserStakeModuleImpl)]
    fn user_stake(&self) -> UserStakeModuleImpl<T, BigInt, BigUint>;

    #[module(UserDataModuleImpl)]
    fn user_data(&self) -> UserDataModuleImpl<T, BigInt, BigUint>;

    #[module(NodeConfigModuleImpl)]
    fn node_config(&self) -> NodeConfigModuleImpl<T, BigInt, BigUint>;

    #[module(NodeActivationModuleImpl)]
    fn node_activation(&self) -> NodeActivationModuleImpl<T, BigInt, BigUint>;


    /// Function to be used only during genesis block.
    /// Cannot perform payments during genesis block, so we update state but not the balance.
    fn stakeGenesis(&self, stake: BigUint) -> Result<(), &str> {
        if self.get_block_nonce() > 0 {
            return Err("genesis block only")
        }
        self.user_stake()._process_stake(stake)
    }

    /// Function to be used only once, during genesis block.
    /// Cannot perform payments during genesis block, so we update state but do not receive or send funds.
    fn activateGenesis(&self) -> Result<(), &str> {
        if self.get_block_nonce() > 0 {
            return Err("genesis block only")
        }

        // set nodes to Active, and count how many not deleted
        let num_nodes = self.node_config().getNumNodes();
        let mut num_inactive_nodes = 0usize;
        for node_id in 1..num_nodes {
            match self.node_config().getNodeState(node_id) {
                NodeState::Inactive => {
                    self.node_config()._set_node_state(node_id, NodeState::Active);
                    num_inactive_nodes += 1;
                },
                NodeState::Removed => {},
                _ => {
                    return Err("cannot activate twice during genesis");
                },
            }
        }

        // validate that node stake and user stake match
        let stake_required_by_nodes = BigUint::from(num_inactive_nodes) * self.node_config().getStakePerNode();
        let total_inactive_stake = self.user_data()._get_user_stake_of_type(USER_STAKE_TOTALS_ID, UserStakeState::Inactive);
        if stake_required_by_nodes != total_inactive_stake {
            return Err("stake required by nodes must match total user stake at genesis");
        }

        // convert all user inactive stake to active stake
        self.user_data().transform_user_stake_asc(UserStakeState::Inactive, UserStakeState::Active, &total_inactive_stake)?;

        // log event (no data)
        self.events().activation_ok_event(());

        Ok(())
    }

}