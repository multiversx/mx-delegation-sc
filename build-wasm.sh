#!/bin/sh

erdpy --verbose contract build "/home/andreim/elrond/newsc/sc-delegation-rs/v0_5_full"
erdpy --verbose contract build "/home/andreim/elrond/newsc/sc-delegation-rs/v0_5_update"



## For playing around without erdpy:

# cd v0_5/wasm
# RUSTFLAGS='-C link-arg=-s' \
# cargo build --target=wasm32-unknown-unknown --release
# cd ..
# mkdir -p output
# cp wasm/target/wasm32-unknown-unknown/release/delegation_v0_5_wasm.wasm output/delegation.wasm
# cd ..

# cd auction-mock/wasm
# RUSTFLAGS='-C link-arg=-s' \
# cargo build --target=wasm32-unknown-unknown --release
# cd ..
# mkdir -p output
# cp wasm/target/wasm32-unknown-unknown/release/auction_mock_wasm.wasm output/auction-mock.wasm
# cd ..
