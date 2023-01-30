#!/bin/sh

# cleans all wasm targets

sc-meta all clean

# not wasm, but worth cleaning from time to time

cargo clean
