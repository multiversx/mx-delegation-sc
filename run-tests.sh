#!/bin/sh

# Works with the latest version of Arwen cloned in the default location
# Will be replaced by erdpy
$GOPATH/src/github.com/ElrondNetwork/arwen-wasm-vm/cmd/test/test test

# erdpy command examples:

# erdpy test
# erdpy --verbose test  --directory="test/init/init.scen.json"
