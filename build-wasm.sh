#!/bin/sh

DELEGATION_DIR=$(dirname "$0")
erdpy --verbose contract build "$DELEGATION_DIR/latest_full"
erdpy --verbose contract build "$DELEGATION_DIR/latest_update"

cp latest_full/output/delegation_latest_full.wasm v0_5_6_full/output/delegation_v0_5_6_full.wasm
cp latest_update/output/delegation_latest_update.wasm v0_5_6_update/output/delegation_v0_5_6_update.wasm

## For playing around without erdpy:

# cd latest/wasm
# RUSTFLAGS='-C link-arg=-s' \
# cargo build --target=wasm32-unknown-unknown --release
# cd ..
# mkdir -p output
# cp wasm/target/wasm32-unknown-unknown/release/delegation_latest_wasm.wasm output/delegation.wasm
# cd ..

# cd auction-mock/wasm
# RUSTFLAGS='-C link-arg=-s' \
# cargo build --target=wasm32-unknown-unknown --release
# cd ..
# mkdir -p output
# cp wasm/target/wasm32-unknown-unknown/release/auction_mock_wasm.wasm output/auction-mock.wasm
# cd ..
