//! The ranges a run draws from, read out of the config once so that drawing an event
//! does not have to borrow the interactor.

use multiversx_sc_snippets::imports::*;

use crate::test_env::TestConfig;

pub struct Bounds {
    pub num_delegators: usize,
    pub block_advance_max: u64,
    pub max_stake: RustBigUint,
    pub max_reward: RustBigUint,
    pub max_delegation_cap: RustBigUint,
    pub max_dust: RustBigUint,
    pub max_service_fee: usize,
    pub max_nodes_added: usize,
    pub max_nodes_removed: usize,
}

impl Bounds {
    pub fn from_config(config: &TestConfig) -> Self {
        let fuzz = &config.fuzz;

        Bounds {
            num_delegators: config.delegators.count,
            block_advance_max: fuzz.block_advance_max,
            max_stake: fuzz.max_stake(),
            max_reward: fuzz.max_reward(),
            max_delegation_cap: fuzz.max_delegation_cap(),
            max_dust: fuzz.max_dust(),
            max_service_fee: fuzz.max_service_fee,
            max_nodes_added: fuzz.max_nodes_added,
            max_nodes_removed: fuzz.max_nodes_removed,
        }
    }
}
