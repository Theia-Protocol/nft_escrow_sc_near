#!/usr/bin/env bash

WASM_PATH="../target/wasm32-unknown-unknown/release/nft_collection*.wasm"

near dev-deploy \
  --wasmFile $WASM_PATH \
  "$@"

near call "$(<./neardev/dev-account)" new "$(node ./init-args.js)" \
  --accountId "$(<./neardev/dev-account)"
