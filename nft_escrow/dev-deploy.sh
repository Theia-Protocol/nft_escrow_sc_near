#!/usr/bin/env bash

WASM_PATH="$(find ../target/wasm32-unknown-unknown/release/nft_escrow_sc.wasm)"

near dev-deploy \
  --wasmFile $WASM_PATH \
  "$@"

near call "$(<./neardev/dev-account)" new "$(node ./init-args.js)" \
  --accountId "$(<./neardev/dev-account)"
