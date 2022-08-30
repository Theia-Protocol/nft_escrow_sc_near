#!/usr/bin/env bash

export WASM_NAME=nft_escrow_sc.wasm
cargo build --target wasm32-unknown-unknown --release
#wasm-opt -Os -o ../target/wasm32-unknown-unknown/release/$WASM_NAME ../target/wasm32-unknown-unknown/release/$WASM_NAME