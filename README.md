# sc-delegation-rs
Smart contract for managing staking delegation in Elrond.


# Build

Run
```
./compile-wasm.sh 
```
in root, it will produce delegation.wasm and staking-mock.wasm (a dummy used in tests instead of a staking contract).

# Rust configuration

```
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown
rustc --version
cargo install wasm-snip
```

# Advanced

Wasm is built with this line:
```
cargo build --bin wasm --target=wasm32-unknown-unknown --release
```

To debug macros:
- wasm mode:
```
cargo +nightly rustc --bin wasm -- -Z unstable-options --pretty=expanded > demacroed.rs
```

- debug mode:
```
cargo +nightly rustc --lib -- -Z unstable-options --pretty=expanded > demacroed.rs
```

To check wasm size:
```
twiggy top -n 20 target/wasm32-unknown-unknown/release/wasm.wasm
```

To work with unpublished elrond-wasm crates, clone https://github.com/ElrondNetwork/elrond-wasm-rs in the same parent directory and replace dependencies in Cargo.toml with:
```
elrond-wasm = { path = "../../elrond-wasm-rs/elrond-wasm" }
elrond-wasm-node = { path = "../../elrond-wasm-rs/elrond-wasm-node" }
elrond-wasm-derive = { path = "../../elrond-wasm-rs/elrond-wasm-derive" }
```

And the same for debug:
```
elrond-wasm = { path = "../../../elrond-wasm-rs/elrond-wasm" }
elrond-wasm-debug = { path = "../../../elrond-wasm-rs/elrond-wasm-debug" }
```
