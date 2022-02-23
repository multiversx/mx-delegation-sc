use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();
    blockchain.set_current_dir_from_workspace("latest_update");
    blockchain.register_contract_builder(
        "file:../auction-mock/output/auction-mock.wasm",
        auction_mock::ContractBuilder,
    );
    blockchain.register_contract_builder(
        "file:output/delegation_latest_update.wasm",
        delegation_latest_update::ContractBuilder,
    );
    blockchain
}

#[test]
fn genesis_addr_fix() {
    elrond_wasm_debug::mandos_rs("mandos/genesis_addr_fix.scen.json", world());
}

#[test]
fn version_rs() {
    elrond_wasm_debug::mandos_rs("mandos/version.scen.json", world());
}
