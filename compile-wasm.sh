#!/bin/sh

# script provided for convenience, to build and extract wasm output to root

rm output/delegation.wasm
rm test/delegation.wasm
rm output/auction-mock.wasm
rm test/auction-mock.wasm

RUSTFLAGS='-C link-arg=-s' \
cargo build --bin delegation --target=wasm32-unknown-unknown --release
mkdir -p output

cp target/wasm32-unknown-unknown/release/delegation.wasm output/delegation.wasm
cp target/wasm32-unknown-unknown/release/delegation.wasm test/delegation.wasm
rm target/wasm32-unknown-unknown/release/delegation.wasm
# wasm-snip output/delegation.wasm -o output/delegation.wasm --snip-rust-fmt-code --snip-rust-panicking-code
# twiggy top -n 20 output/delegation.wasm
# twiggy top -n 300 delegation.wasm > twiggy-snip.txt

cd auction-mock
cargo build --bin auction-mock --target=wasm32-unknown-unknown --release
cd ..
cp auction-mock/target/wasm32-unknown-unknown/release/auction-mock.wasm output/auction-mock.wasm
cp auction-mock/target/wasm32-unknown-unknown/release/auction-mock.wasm test/auction-mock.wasm
rm auction-mock/target/wasm32-unknown-unknown/release/auction-mock.wasm

# for debugging macros:
# cargo +nightly rustc --lib -- -Z unstable-options --pretty=expanded > demacroed.rs
