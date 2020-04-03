#!/bin/sh

# script provided for convenience, to build and extract wasm output to root

cargo build --bin wasm --target=wasm32-unknown-unknown --release
mv target/wasm32-unknown-unknown/release/wasm.wasm delegation.wasm
wasm-snip delegation.wasm -o delegation.wasm --snip-rust-fmt-code --snip-rust-panicking-code
#wasm-gc delegation.wasm
# twiggy top -n 100 delegation.wasm > twiggy-snip.txt

cd auction-mock
cargo build --bin wasm --target=wasm32-unknown-unknown --release
cd ..
mv auction-mock/target/wasm32-unknown-unknown/release/wasm.wasm auction-mock.wasm
