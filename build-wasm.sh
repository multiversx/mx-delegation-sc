#!/bin/sh

sc-meta all build

cp latest_full/output/delegation_latest_full.wasm v0_5_8_full/output/delegation_v0_5_8_full.wasm
cp latest_update/output/delegation_latest_update.wasm v0_5_8_update/output/delegation_v0_5_8_update.wasm
