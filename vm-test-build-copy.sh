#!/bin/bash

# copies wasm & scenarios files to the VM test folder
# expects 1 argument: the path to the `mx-chain-vm-go` repo root

VM_REPO_PATH=${1:?"Missing VM repo path!"}

sc-meta all build

cp latest/output/delegation_latest_full.wasm \
   $VM_REPO_PATH/test/delegation/v0_5_latest/output
cp latest/output/delegation_latest_update.wasm \
   $VM_REPO_PATH/test/delegation/v0_5_latest/output
cp -R latest/scenarios/ \
   $VM_REPO_PATH/test/delegation/v0_5_latest/

cp auction-mock/output/auction-mock.wasm \
   $VM_REPO_PATH/test/delegation/auction-mock/output/auction-mock.wasm

cd $VM_REPO_PATH/integrationTests/json
go test -run v0_5
