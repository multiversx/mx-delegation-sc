#!/bin/sh

# script provided for convenience, to build and extract wasm output to root

cargo build --bin wasm --target=wasm32-unknown-unknown --release
mv target/wasm32-unknown-unknown/release/wasm.wasm delegation.wasm

cd staking-mock
cargo build --bin wasm --target=wasm32-unknown-unknown --release
cd ..
mv staking-mock/target/wasm32-unknown-unknown/release/wasm.wasm staking-mock.wasm
