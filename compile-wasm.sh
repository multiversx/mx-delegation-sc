#!/bin/sh

# script provided for convenience, to build and extract wasm output to root

#RUSTFLAGS='-C link-arg=-s' \
cargo build --bin delegation --target=wasm32-unknown-unknown --release
mkdir -p output

mv target/wasm32-unknown-unknown/release/delegation.wasm output/delegation.wasm
wasm-snip output/delegation.wasm -o output/delegation.wasm --snip-rust-fmt-code --snip-rust-panicking-code
# twiggy top -n 20 output/delegation.wasm
# twiggy top -n 300 delegation.wasm > twiggy-snip.txt

cd auction-mock
cargo build --bin auction-mock --target=wasm32-unknown-unknown --release
cd ..
mv auction-mock/target/wasm32-unknown-unknown/release/auction-mock.wasm output/auction-mock.wasm
