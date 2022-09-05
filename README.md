# nft_escrow_sc_near
NFT escrow smart contract on Near network

# Required Software

- Rust 1.61 + cargo
- Node.js
- NEAR CLI 3.2

# Usage

## Scripts

### `build.sh`

Compiles the smart contract to a WebAssembly binary. The binary path is `./target/wasm32-unknown-unknown/release/near_smart_contract_rust_template.wasm`.

### `contract.sh <command> <...arguments>`

Calls the NEAR CLI, where `<dev-account>` is the account ID of the most recent dev deployment on testnet:

```txt
near <command> <dev-account> <...arguments>
```

### `deploy.sh <account-id>`

Deploys the most recently built WASM binary to `<account-id>` on mainnet, and calls the `new` function with arguments generated by `init-args.js`.

### `dev-deploy.sh [--force]`

Deploys the most recently built WASM binary to the dev account in `neardev/`, or to a new dev account if `neardev/` is not found or `--force` is set. Calls the `new` function with arguments generated by `init-args.js`.
