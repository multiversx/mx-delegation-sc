#[test]
fn activate_nodes_go() {
    multiversx_sc_scenario::run_go("scenarios/activate_nodes.scen.json");
}

#[test]
fn change_service_fee_go() {
    multiversx_sc_scenario::run_go("scenarios/change_service_fee.scen.json");
}

#[test]
fn claim_rewards_1_go() {
    multiversx_sc_scenario::run_go("scenarios/claim_rewards_1.scen.json");
}

#[test]
fn claim_rewards_owner_with_stake_go() {
    multiversx_sc_scenario::run_go("scenarios/claim_rewards_owner_with_stake.scen.json");
}

#[test]
fn claim_rewards_with_changed_service_fee_go() {
    multiversx_sc_scenario::run_go("scenarios/claim_rewards_with_changed_service_fee.scen.json");
}

#[test]
fn claim_rewards_with_modify_delegation_cap_go() {
    multiversx_sc_scenario::run_go("scenarios/claim_rewards_with_modify_delegation_cap.scen.json");
}

#[test]
fn claim_rewards_with_stake_modifications_go() {
    multiversx_sc_scenario::run_go("scenarios/claim_rewards_with_stake_modifications.scen.json");
}

#[test]
fn continue_global_operations_go() {
    multiversx_sc_scenario::run_go("scenarios/continue_global_operations.scen.json");
}

#[test]
fn decrease_cap_in_bootstrap_mode_go() {
    multiversx_sc_scenario::run_go("scenarios/decrease_cap_in_bootstrap_mode.scen.json");
}

#[test]
fn increase_delegation_cap_go() {
    multiversx_sc_scenario::run_go("scenarios/increase_delegation_cap.scen.json");
}

#[test]
fn rewards_for_unstaked_go_to_the_owner_go() {
    multiversx_sc_scenario::run_go("scenarios/rewards_for_unStaked_go_to_the_owner.scen.json");
}

#[test]
fn set_num_blocks_before_unbond_go() {
    multiversx_sc_scenario::run_go("scenarios/set_num_blocks_before_unbond.scen.json");
}

#[test]
fn staking_1_go() {
    multiversx_sc_scenario::run_go("scenarios/staking_1.scen.json");
}

#[test]
fn staking_2_go() {
    multiversx_sc_scenario::run_go("scenarios/staking_2.scen.json");
}

#[test]
fn total_funds_getters_go() {
    multiversx_sc_scenario::run_go("scenarios/total_funds_getters.scen.json");
}

#[test]
fn unbond_go() {
    multiversx_sc_scenario::run_go("scenarios/unbond.scen.json");
}

#[test]
fn unbond_from_waiting_go() {
    multiversx_sc_scenario::run_go("scenarios/unbond_from_waiting.scen.json");
}

#[test]
fn unjail_go() {
    multiversx_sc_scenario::run_go("scenarios/unjail.scen.json");
}

#[test]
fn unstake_1_go() {
    multiversx_sc_scenario::run_go("scenarios/unstake_1.scen.json");
}

#[test]
fn unstake_2_go() {
    multiversx_sc_scenario::run_go("scenarios/unstake_2.scen.json");
}

#[test]
fn unstake_3_go() {
    multiversx_sc_scenario::run_go("scenarios/unstake_3.scen.json");
}

#[test]
fn unstake_4_go() {
    multiversx_sc_scenario::run_go("scenarios/unstake_4.scen.json");
}

#[test]
fn unstake_5_backwards_go() {
    multiversx_sc_scenario::run_go("scenarios/unstake_5_backwards.scen.json");
}

#[test]
fn user_fund_getters_go() {
    multiversx_sc_scenario::run_go("scenarios/user_fund_getters.scen.json");
}

#[test]
fn version_go() {
    multiversx_sc_scenario::run_go("scenarios/version.scen.json");
}
