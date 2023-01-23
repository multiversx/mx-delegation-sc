#[test]
fn genesis_addr_fix_go() {
    multiversx_sc_scenario::run_go("mandos/genesis_addr_fix.scen.json");
}

#[test]
fn version_go() {
    multiversx_sc_scenario::run_go("mandos/version.scen.json");
}
