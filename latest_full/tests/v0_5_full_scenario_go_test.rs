#[test]
fn activate_nodes_go() {
    multiversx_sc_scenario::run_go("mandos/activate_nodes.scen.json");
}

#[test]
fn change_service_fee_go() {
    multiversx_sc_scenario::run_go("mandos/change_service_fee.scen.json");
}

#[test]
fn claim_rewards_1_go() {
    multiversx_sc_scenario::run_go("mandos/claim_rewards_1.scen.json");
}

#[test]
fn claim_rewards_owner_with_stake_go() {
    multiversx_sc_scenario::run_go("mandos/claim_rewards_owner_with_stake.scen.json");
}

#[test]
fn claim_rewards_with_changed_service_fee_go() {
    multiversx_sc_scenario::run_go("mandos/claim_rewards_with_changed_service_fee.scen.json");
}

#[test]
fn claim_rewards_with_modify_delegation_cap_go() {
    multiversx_sc_scenario::run_go("mandos/claim_rewards_with_modify_delegation_cap.scen.json");
}

#[test]
fn claim_rewards_with_stake_modifications_go() {
    multiversx_sc_scenario::run_go("mandos/claim_rewards_with_stake_modifications.scen.json");
}

#[test]
fn continue_global_operations_go() {
    multiversx_sc_scenario::run_go("mandos/continue_global_operations.scen.json");
}

#[test]
fn decrease_cap_in_bootstrap_mode_go() {
    multiversx_sc_scenario::run_go("mandos/decrease_cap_in_bootstrap_mode.scen.json");
}

#[test]
fn increase_delegation_cap_go() {
    multiversx_sc_scenario::run_go("mandos/increase_delegation_cap.scen.json");
}

#[test]
fn rewards_for_unstaked_go_to_the_owner_go() {
    multiversx_sc_scenario::run_go("mandos/rewards_for_unStaked_go_to_the_owner.scen.json");
}

#[test]
fn set_num_blocks_before_unbond_go() {
    multiversx_sc_scenario::run_go("mandos/set_num_blocks_before_unbond.scen.json");
}

#[test]
fn staking_1_go() {
    multiversx_sc_scenario::run_go("mandos/staking_1.scen.json");
}

#[test]
fn staking_2_go() {
    multiversx_sc_scenario::run_go("mandos/staking_2.scen.json");
}

#[test]
fn total_funds_getters_go() {
    multiversx_sc_scenario::run_go("mandos/total_funds_getters.scen.json");
}

#[test]
fn unbond_go() {
    multiversx_sc_scenario::run_go("mandos/unbond.scen.json");
}

#[test]
fn unbond_from_waiting_go() {
    multiversx_sc_scenario::run_go("mandos/unbond_from_waiting.scen.json");
}

#[test]
fn unjail_go() {
    multiversx_sc_scenario::run_go("mandos/unjail.scen.json");
}

#[test]
fn unstake_1_go() {
    multiversx_sc_scenario::run_go("mandos/unstake_1.scen.json");
}

#[test]
fn unstake_2_go() {
    multiversx_sc_scenario::run_go("mandos/unstake_2.scen.json");
}

#[test]
fn unstake_3_go() {
    multiversx_sc_scenario::run_go("mandos/unstake_3.scen.json");
}

#[test]
fn unstake_4_go() {
    multiversx_sc_scenario::run_go("mandos/unstake_4.scen.json");
}

#[test]
fn unstake_5_backwards_go() {
    multiversx_sc_scenario::run_go("mandos/unstake_5_backwards.scen.json");
}

#[test]
fn user_fund_getters_go() {
    multiversx_sc_scenario::run_go("mandos/user_fund_getters.scen.json");
}

#[test]
fn version_go() {
    multiversx_sc_scenario::run_go("mandos/version.scen.json");
}
