use elrond_wasm::*;
use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();
    blockchain.set_current_dir_from_workspace("latest_update");
    blockchain.register_contract(
        "file:../auction-mock/output/auction-mock.wasm",
        Box::new(|context| Box::new(auction_mock::contract_obj(context))),
    );

    blockchain.register_contract(
        "file:output/delegation_latest_update.wasm",
        Box::new(|context| Box::new(delegation_latest_update::contract_obj(context))),
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
