mod helpers;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::log_str;
use near_sdk::serde::{Deserialize, Serialize};
use serde_json::json;
use workspaces::prelude::*;
use workspaces::{Account, Contract, DevNetwork, Worker};
use helpers::*;

const FUNGIBLE_TOKEN_CODE: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/ft_token.wasm");
const NFT_ESCROW_CODE: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/nft_escrow_sc.wasm");
// const PROXY_TOKEN_CODE: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/proxy_token.wasm");
// const NFT_COLLECTION_CODE: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/nft_collection.wasm");

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
enum CurveType {
    Horizontal,
    Linear,
    Sigmoidal,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
struct CurveArgs {
    pub arg_a: Option<u128>,
    pub arg_b: Option<u128>,
    pub arg_c: Option<u128>,
    pub arg_d: Option<u128>,
}

async fn init(
    worker: &Worker<impl DevNetwork>
) -> anyhow::Result<(Contract, Contract, Account, Account, Account, Account, Account)> {
    let owner = worker.dev_create_account().await?;
    let alice = worker.dev_create_account().await?;
    let bob = worker.dev_create_account().await?;
    let finder = worker.dev_create_account().await?;
    let treasury = worker.dev_create_account().await?;

    let stable_coin_contract = worker.dev_deploy(FUNGIBLE_TOKEN_CODE).await?;
    let res = stable_coin_contract
        .call(&worker, "new")
        .args_json((owner.id(), String::from("USD Tether"), String::from("USDT")))?
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    let escrow_contract = worker.dev_deploy(NFT_ESCROW_CODE).await?;
    let curve_args = CurveArgs {
        arg_a: Some(100u128),
        arg_b: None,
        arg_c: None,
        arg_d: None,
    };

    let res = escrow_contract
        .call(&worker, "new")
        .args_json((owner.id(), stable_coin_contract.id(), 24u8, CurveType::Horizontal, curve_args, treasury.id()))?
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    Ok((escrow_contract, stable_coin_contract, owner, alice, bob, finder, treasury))
}


#[tokio::test]
async fn test_active_nft_project() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (contract, _, owner, _, _, finder, _) = init(&worker).await?;

    log_str(format!("owner: {}", owner.id()).as_str());

    // let res = owner
    //     .call(&worker, contract.id(), "active_nft_project".into())
    //     .args_json((NAME, SYMBOL, NFT_BASE_URI, NFT_BLANK_URI, NFT_MAX_SUPPLY, finder.id(), PRE_MINT_AMOUNT, FUND_THRESHOLD, ONE_DAY, TWO_DAYS))?
    //     .max_gas()
    //     .transact()
    //     .await?;
    // assert!(res.is_success());

    Ok(())
}