#!/usr/bin/env bash

export WASM_NAME=proxy_token.wasm
cargo build --target wasm32-unknown-unknown --release
wasm-opt -Os -o ../target/wasm32-unknown-unknown/release/$WASM_NAME ../target/wasm32-unknown-unknown/release/$WASM_NAME
