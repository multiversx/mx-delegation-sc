#!/bin/sh

# until we have the new version of erdpy

cd v0_5/wasm
RUSTFLAGS='-C link-arg=-s' \
cargo build --target=wasm32-unknown-unknown --release
cd ..
mkdir -p output
cp wasm/target/wasm32-unknown-unknown/release/delegation_v0_5_wasm.wasm output/delegation.wasm
cd ..
