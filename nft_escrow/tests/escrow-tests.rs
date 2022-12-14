mod helpers;

use near_sdk::json_types::U128;
use serde_json::json;
use workspaces::prelude::*;
use workspaces::{Account, Contract, DevNetwork, Worker, AccountId};
use helpers::*;

const FUNGIBLE_TOKEN_CODE: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/ft_token.wasm");
const NFT_ESCROW_CODE: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/nft_escrow_sc.wasm");
const STORAGE_BYTE_COST: u128 = 10_000_000_000_000_000_000;
const DEPOSIT_ONE_PT_MINT: u128 = 415 * STORAGE_BYTE_COST;

fn parse_unit_with_decimals(amount: u128, decimals: u8) -> u128 {
    return amount * 10u128.pow(decimals as u32)
}

async fn init(
    worker: &Worker<impl DevNetwork>
) -> anyhow::Result<(Contract, Contract, Account, Account, Account, Account, Account, u128)> {
    let owner = worker.dev_create_account().await?;
    let alice = worker.dev_create_account().await?;
    let bob = worker.dev_create_account().await?;
    let finder = worker.dev_create_account().await?;
    let treasury = worker.dev_create_account().await?;

    let stable_coin_decimals = 24u8;
    let one_coin = 10u128.pow(stable_coin_decimals as u32);

    // deploy USDT
    let stable_coin_contract = worker.dev_deploy(FUNGIBLE_TOKEN_CODE).await?;
    let res = stable_coin_contract
        .call(&worker, "new")
        .args_json((owner.id(), String::from("USD Tether"), String::from("USDT"), 24u8))?
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    // register owner
    stable_coin_contract
        .call(&worker, "storage_deposit")
        .args_json((owner.id(), Option::<bool>::None))?
        .deposit(125 * STORAGE_BYTE_COST)
        .max_gas()
        .transact()
        .await?;
        
    // treansfer 1000 USDT to owner
    owner.call(&worker, stable_coin_contract.id(), "ft_mint")
        .args_json((owner.id(), U128(parse_unit_with_decimals(1000_000_000u128, 24u8))))?
        .max_gas()
        .transact()
        .await?;

    // register alice
    stable_coin_contract
        .call(&worker, "storage_deposit")
        .args_json((alice.id(), Option::<bool>::None))?
        .deposit(125 * STORAGE_BYTE_COST)
        .max_gas()
        .transact()
        .await?;

    // treansfer 1000 USDT to alice
    owner.call(&worker, stable_coin_contract.id(), "ft_mint")
        .args_json((alice.id(), U128(parse_unit_with_decimals(1000_000_000u128, 24u8))))?
        .max_gas()
        .transact()
        .await?;

    // register finder
    stable_coin_contract
        .call(&worker, "storage_deposit")
        .args_json((finder.id(), Option::<bool>::None))?
        .deposit(125 * STORAGE_BYTE_COST)
        .max_gas()
        .transact()
        .await?;

    // register treasury
    stable_coin_contract
        .call(&worker, "storage_deposit")
        .args_json((treasury.id(), Option::<bool>::None))?
        .deposit(125 * STORAGE_BYTE_COST)
        .max_gas()
        .transact()
        .await?;


    // deploy escrow
    let escrow_contract = worker.dev_deploy(NFT_ESCROW_CODE).await?;
    let curve_args = CurveArgs {
        arg_a: Some(100u128),
        arg_b: None,
        arg_c: None,
        arg_d: None,
    };

    // register escrow contract
    let res = escrow_contract
        .call(&worker, "new")
        .args_json((owner.id(), NAME, SYMBOL, NFT_BLANK_URI, stable_coin_contract.id(), stable_coin_decimals, CurveType::Horizontal, curve_args, treasury.id()))?
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());


    stable_coin_contract
        .call(&worker, "storage_deposit")
        .args_json((escrow_contract.id(), Option::<bool>::None))?
        .deposit(125 * STORAGE_BYTE_COST)
        .max_gas()
        .transact()
        .await?;

    Ok((escrow_contract, stable_coin_contract, owner, alice, bob, finder, treasury, one_coin))
}

#[tokio::test]
async fn test_active_nft_project() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (contract, _, owner, _, _, finder, _, _) = init(&worker).await?;

    let res = owner
        .call(&worker, contract.id(), "active_nft_project".into())
        .args_json((NFT_BASE_URI, NFT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success() && res.failures().is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_active_ft_project() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (contract, _, owner, _, _, finder, _, _) = init(&worker).await?;

    let res = owner
        .call(&worker, contract.id(), "active_ft_project".into())
        .args_json((FT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success() && res.failures().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_auction_curve_horizontal() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (_, stable_coin_contract, owner, _, _, finder, treasury, one_coin) = init(&worker).await?;


    // deploy
    let escrow_contract = worker.dev_deploy(NFT_ESCROW_CODE).await?;
    const BASE_TOKEN_PRICE: u128 = 100u128;
    let curve_args = CurveArgs {
        arg_a: Some(BASE_TOKEN_PRICE),
        arg_b: None,
        arg_c: None,
        arg_d: None,
    };

    // initialize
    escrow_contract
        .call(&worker, "new")
        .args_json((owner.id(), NAME, SYMBOL, NFT_BLANK_URI, stable_coin_contract.id(), 24u8, CurveType::Horizontal, &curve_args, treasury.id()))?
        .max_gas()
        .transact()
        .await?;

    // active project
    owner
        .call(&worker, escrow_contract.id(), "active_nft_project")
        .args_json((NFT_BASE_URI, NFT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;

    let curve_type = escrow_contract.call(&worker, "get_curve_type")
        .view()
        .await?
        .json::<CurveType>()?;

    assert_eq!(curve_type, CurveType::Horizontal);

    let _curve_args = escrow_contract.call(&worker, "get_curve_args")
        .view()
        .await?
        .json::<CurveArgs>()?;

    assert_eq!(&_curve_args.arg_a, &curve_args.arg_a);

    for token_id in 1..10 {
        let token_price =
            escrow_contract
                .view(
                    &worker,
                    "get_token_price",
                    json!({
                        "token_id": U128::from(token_id as u128 + PRE_MINT_AMOUNT.0) 
                    }).to_string().into_bytes(),
                )
                .await?
                .json::<u128>()?;

        println!("Token ID: {}, Curve Price: {}", token_id.to_string(), (token_price / one_coin).to_string());
    }

    println!("-- Buy Price --");
    for amount in 0..10 {
        let buy_price =
            escrow_contract
                .view(
                    &worker,
                    "calculate_buy_proxy_token",
                    json!({
                        "amount": U128::from(amount as u128)
                    }).to_string().into_bytes(),
                )
                .await?
                .json::<u128>()?;

        println!("Amount: {}, Buy Price: {}", amount.to_string(), (buy_price as f64 / one_coin as f64).to_string());
    }

    println!("-- Sell Price --");
    for token_id in 0..10 {
        let token_ids: Vec<String> = vec![(token_id + PRE_MINT_AMOUNT.0).to_string()];
        let buy_price =
            escrow_contract
                .view(
                    &worker,
                    "calculate_sell_proxy_token",
                    json!({
                        "token_ids": token_ids
                    }).to_string().into_bytes(),
                )
                .await?
                .json::<u128>()?;

        println!("Token ID: {}, Sell Price: {}", token_id.to_string(), (buy_price as f64 / one_coin as f64).to_string());
    }

    Ok(())
}

#[tokio::test]
async fn test_auction_curve_linear() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (_, stable_coin_contract, owner, _, _, finder, treasury, one_coin) = init(&worker).await?;


    // deploy
    let escrow_contract = worker.dev_deploy(NFT_ESCROW_CODE).await?;
    const CURVE_K: u128 = 2u128;
    const BASE_TOKEN_PRICE: u128 = 100u128;
    let curve_args = CurveArgs {
        arg_a: Some(CURVE_K),
        arg_b: Some(BASE_TOKEN_PRICE),
        arg_c: None,
        arg_d: None,
    };

    // initialize
    escrow_contract
        .call(&worker, "new")
        .args_json((owner.id(), NAME, SYMBOL, NFT_BLANK_URI, stable_coin_contract.id(), 24u8, CurveType::Linear, &curve_args, treasury.id()))?
        .max_gas()
        .transact()
        .await?;

    // active project
    owner
        .call(&worker, escrow_contract.id(), "active_nft_project")
        .args_json((NFT_BASE_URI, NFT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;

    let curve_type = escrow_contract.call(&worker, "get_curve_type")
        .view()
        .await?
        .json::<CurveType>()?;

    assert_eq!(curve_type, CurveType::Linear);

    let _curve_args = escrow_contract.call(&worker, "get_curve_args")
        .view()
        .await?
        .json::<CurveArgs>()?;

    assert_eq!(&_curve_args.arg_a, &curve_args.arg_a);
    assert_eq!(&_curve_args.arg_b, &curve_args.arg_b);

    println!("-- Token Price --");
    for token_id in 0..10 {
        let token_price =
            escrow_contract
                .view(
                    &worker,
                    "get_token_price",
                    json!({
                        "token_id": U128::from(token_id as u128 + PRE_MINT_AMOUNT.0)
                    }).to_string().into_bytes(),
                )
                .await?
                .json::<u128>()?;

        println!("Token ID: {}, Curve Price: {}", token_id.to_string(), (token_price / one_coin).to_string());
    }

    println!("-- Buy Price --");
    for amount in 0..10 {
        let buy_price =
            escrow_contract
                .view(
                    &worker,
                    "calculate_buy_proxy_token",
                    json!({
                        "amount": U128::from(amount as u128)
                    }).to_string().into_bytes(),
                )
                .await?
                .json::<u128>()?;

        println!("Amount: {}, Buy Price: {}", amount.to_string(), (buy_price as f64 / one_coin as f64).to_string());
    }

    println!("-- Sell Price --");
    for token_id in 0..10 {
        let token_ids: Vec<String> = vec![(token_id + PRE_MINT_AMOUNT.0).to_string()];
        let buy_price =
            escrow_contract
                .view(
                    &worker,
                    "calculate_sell_proxy_token",
                    json!({
                        "token_ids": token_ids
                    }).to_string().into_bytes(),
                )
                .await?
                .json::<u128>()?;

        println!("Token ID: {}, Sell Price: {}", token_id.to_string(), (buy_price as f64 / one_coin as f64).to_string());
    }

    Ok(())
}

#[tokio::test]
async fn test_auction_curve_sigmoidal() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (_, stable_coin_contract, owner, _, _, finder, treasury, one_coin) = init(&worker).await?;


    // deploy
    let escrow_contract = worker.dev_deploy(NFT_ESCROW_CODE).await?;
    const CURVE_K: u128 = 30u128;
    const ARG_B: u128 = 10u128;
    const ARG_C: u128 = 100u128;
    const BASE_TOKEN_PRICE: u128 = 100u128;
    let curve_args = CurveArgs {
        arg_a: Some(CURVE_K),
        arg_b: Some(ARG_B),
        arg_c: Some(ARG_C),
        arg_d: Some(BASE_TOKEN_PRICE),
    };

    // initialize
    escrow_contract
        .call(&worker, "new")
        .args_json((owner.id(), NAME, SYMBOL, NFT_BLANK_URI, stable_coin_contract.id(), 24u8, CurveType::Sigmoidal, &curve_args, treasury.id()))?
        .max_gas()
        .transact()
        .await?;

    // active project
    owner
        .call(&worker, escrow_contract.id(), "active_nft_project")
        .args_json((NFT_BASE_URI, NFT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;

    let curve_type = escrow_contract.call(&worker, "get_curve_type")
        .view()
        .await?
        .json::<CurveType>()?;

    assert_eq!(curve_type, CurveType::Sigmoidal);

    let _curve_args = escrow_contract.call(&worker, "get_curve_args")
        .view()
        .await?
        .json::<CurveArgs>()?;

    assert_eq!(&_curve_args.arg_a, &curve_args.arg_a);
    assert_eq!(&_curve_args.arg_b, &curve_args.arg_b);
    assert_eq!(&_curve_args.arg_c, &curve_args.arg_c);
    assert_eq!(&_curve_args.arg_d, &curve_args.arg_d);

    println!("-- Token Price --");
    for token_id in 0..10 {
        let token_price =
            escrow_contract
                .view(
                    &worker,
                    "get_token_price",
                    json!({
                        "token_id": U128::from(token_id as u128 + PRE_MINT_AMOUNT.0)
                    }).to_string().into_bytes(),
                )
                .await?
                .json::<u128>()?;

        println!("Token ID: {}, Curve Price: {}", token_id.to_string(), (token_price as f64 / one_coin as f64).to_string());
    }

    println!("-- Buy Price --");
    for amount in 0..10 {
        let buy_price =
            escrow_contract
                .view(
                    &worker,
                    "calculate_buy_proxy_token",
                    json!({
                        "amount": U128::from(amount as u128)
                    }).to_string().into_bytes(),
                )
                .await?
                .json::<u128>()?;

        println!("Amount: {}, Buy Price: {}", amount.to_string(), (buy_price as f64 / one_coin as f64).to_string());
    }

    println!("-- Sell Price --");
    for token_id in 0..10 {
        let token_ids: Vec<String> = vec![(token_id + PRE_MINT_AMOUNT.0).to_string()];
        let buy_price =
            escrow_contract
                .view(
                    &worker,
                    "calculate_sell_proxy_token",
                    json!({
                        "token_ids": token_ids
                    }).to_string().into_bytes(),
                )
                .await?
                .json::<u128>()?;

        println!("Token ID: {}, Sell Price: {}", token_id.to_string(), (buy_price as f64 / one_coin as f64).to_string());
    }

    Ok(())
}

#[tokio::test]
async fn test_revert_nft_buy() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (escrow_contract, stable_coin_contract, owner, _, _, finder, treasury, _) = init(&worker).await?;

    // active project
    let _res = owner
        .call(&worker, escrow_contract.id(), "active_nft_project")
        .args_json((NFT_BASE_URI, NFT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;
    // println!("active: {:?}", _res);

    // calculate stable coin amount for buying proxy token
    let amount = U128::from(300u128);
    let coin_amount = escrow_contract
        .view(
            &worker,
            "calculate_buy_proxy_token",
            json!({
            "amount": amount
        }).to_string().into_bytes(),
        )
        .await?
        .json::<u128>()?;

    // buy proxy token
    let res = owner
        .call(&worker, stable_coin_contract.id(), "ft_transfer_call".into())
        .args_json((escrow_contract.id(), U128(coin_amount), Option::<String>::None, format!("buy:{}", amount.0)))?
        .deposit(1u128)
        .max_gas()
        .transact()
        .await?;

    println!("buy: {:?}", res);
    assert!(res.is_success() && !res.failures().is_empty());

    let balance = stable_coin_contract
        .view(
            &worker,
            "ft_balance_of",
            json!({
                "account_id": escrow_contract.id()
            }).to_string().into_bytes()
        )
        .await?
        .json::<U128>()?;
    assert_eq!(balance.0, 0);

    let balance = stable_coin_contract
        .view(
            &worker,
            "ft_balance_of",
            json!({
                "account_id": treasury.id()
            }).to_string().into_bytes()
        )
        .await?
        .json::<U128>()?;
    assert_eq!(balance.0, 0);

    Ok(())
}

#[tokio::test]
async fn test_nft_buy() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (escrow_contract, stable_coin_contract, owner, _, _, finder, treasury, _) = init(&worker).await?;

    // active project
    let _res = owner
        .call(&worker, escrow_contract.id(), "active_nft_project")
        .args_json((NFT_BASE_URI, NFT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;
    // println!("active: {:?}", _res);

    // calculate stable coin amount for buying proxy token
    let amount = U128::from(200u128);
    let coin_amount = escrow_contract
        .view(
            &worker,
            "calculate_buy_proxy_token",
            json!({
            "amount": amount
        }).to_string().into_bytes(),
        )
        .await?
        .json::<u128>()?;

    // buy proxy token
    let res = owner
        .call(&worker, stable_coin_contract.id(), "ft_transfer_call".into())
        .args_json((escrow_contract.id(), U128(coin_amount), Option::<String>::None, format!("buy:{}", amount.0)))?
        .deposit(1u128)
        .max_gas()
        .transact()
        .await?;

    println!("buy: {:?}", res);
    assert!(res.is_success() && res.failures().is_empty());

    let balance = stable_coin_contract
        .view(
            &worker,
            "ft_balance_of",
            json!({
                "account_id": escrow_contract.id()
            }).to_string().into_bytes()
        )
        .await?
        .json::<U128>()?;
    assert_eq!(balance.0, coin_amount * (100u128 - PROTOCOL_FEE as u128)/100u128);

    let balance = stable_coin_contract
        .view(
            &worker,
            "ft_balance_of",
            json!({
                "account_id": treasury.id()
            }).to_string().into_bytes()
        )
        .await?
        .json::<U128>()?;
    assert_eq!(balance.0, coin_amount * (PROTOCOL_FEE as u128)/100u128);

    Ok(())
}


#[tokio::test]
async fn test_ft_buy() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (escrow_contract, stable_coin_contract, owner, _, _, finder, treasury, _) = init(&worker).await?;

    // active project
    let _res = owner
        .call(&worker, escrow_contract.id(), "active_ft_project")
        .args_json((FT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;
    // println!("active: {:?}", _res);

    // calculate stable coin amount for buying proxy token
    let amount = U128::from(100u128);
    let coin_amount = escrow_contract
        .view(
            &worker,
            "calculate_buy_proxy_token",
            json!({
            "amount": amount
        }).to_string().into_bytes(),
        )
        .await?
        .json::<u128>()?;

    // buy proxy token
    let _res = owner
        .call(&worker, stable_coin_contract.id(), "ft_transfer_call".into())
        .args_json((escrow_contract.id(), U128(coin_amount), Option::<String>::None, format!("buy:{}", amount.0)))?
        .deposit(1u128)
        .max_gas()
        .transact()
        .await?;

    assert!(_res.is_success());
    // println!("buy: {:?}", _res);

    let balance = stable_coin_contract
        .view(
            &worker,
            "ft_balance_of",
            json!({
                "account_id": escrow_contract.id()
            }).to_string().into_bytes()
        )
        .await?
        .json::<U128>()?;
    assert_eq!(balance.0, coin_amount * (100u128 - PROTOCOL_FEE as u128)/100u128);

    let balance = stable_coin_contract
        .view(
            &worker,
            "ft_balance_of",
            json!({
                "account_id": treasury.id()
            }).to_string().into_bytes()
        )
        .await?
        .json::<U128>()?;
    assert_eq!(balance.0, coin_amount * (PROTOCOL_FEE as u128)/100u128);

    Ok(())
}

#[tokio::test]
async fn test_sell() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (escrow_contract, stable_coin_contract, owner, _, _, finder, _, _) = init(&worker).await?;

    // active project
    let _res = owner
        .call(&worker, escrow_contract.id(), "active_nft_project")
        .args_json((NFT_BLANK_URI, NFT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;
    
    // buy proxy token
    // calculate stable coin amount for buying proxy token
    let amount = U128::from(200u128);
    let coin_amount = escrow_contract
        .view(
            &worker,
            "calculate_buy_proxy_token",
            json!({
            "amount": amount
        }).to_string().into_bytes(),
        )
        .await?
        .json::<u128>()?;

    let res = owner
        .call(&worker, stable_coin_contract.id(), "ft_transfer_call".into())
        .args_json((escrow_contract.id(), U128(coin_amount), Option::<String>::None, format!("buy:{}", amount.0)))?
        .deposit(1u128)
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    // println!("buy: {:?}", res);

    assert_eq!(owner
        .call(
            &worker,
            escrow_contract.id(),
            "pt_balance_of",
        )
        .args_json((owner.id(), vec![0.to_string(), 1.to_string()]))?
        .view()
        .await?
        .json::<Vec<u128>>()?, vec![1u128.into(), 1u128.into()]);

    let mut token_ids: Vec<String> = vec![];
    for i in 0..99 {
        token_ids.push(i.to_string());
    }
    // sell proxy token
    let res = owner
        .call(&worker, escrow_contract.id(), "sell".into())
        .args(json!({"token_ids": token_ids}).to_string().as_bytes().to_vec())
        .max_gas()
        .transact()
        .await?;

    assert!(res.is_success());
    // println!("sell: {:?}", res);

    assert_eq!(owner
        .call(
            &worker,
            escrow_contract.id(),
            "pt_balance_of",
        )
        .args_json((owner.id(), vec![0.to_string(), 1.to_string()]))?
        .view()
        .await?
        .json::<Vec<u128>>()?, vec![0u128.into(), 0u128.into()]);

    Ok(())
}


#[tokio::test]
async fn test_nft_convert() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (escrow_contract, stable_coin_contract, owner, _, _, finder, _, _) = init(&worker).await?;

    // active project
    owner
        .call(&worker, escrow_contract.id(), "active_nft_project")
        .args_json((NFT_BLANK_URI, NFT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;

    //buy proxy token
    //calculate stable coin amount for buying proxy token
    let amount = U128::from(200u128);
    let coin_amount = escrow_contract
        .view(
            &worker,
            "calculate_buy_proxy_token",
            json!({
            "amount": amount
        }).to_string().into_bytes(),
        )
        .await?
        .json::<u128>()?;

    owner
        .call(&worker, stable_coin_contract.id(), "ft_transfer_call".into())
        .args_json((escrow_contract.id(), U128(coin_amount), Option::<String>::None, format!("buy:{}", amount.0)))?
        .deposit(1u128)
        .max_gas()
        .transact()
        .await?;


    let mut token_ids: Vec<String> = vec![];
    for i in 0..99 {
        token_ids.push(i.to_string());
    }
    // sell proxy token
    owner
        .call(&worker, escrow_contract.id(), "sell".into())
        .args(json!({"token_ids": token_ids}).to_string().as_bytes().to_vec())
        .max_gas()
        .transact()
        .await?;

    worker.fast_forward(300).await?;

    let mut token_ids: Vec<String> = vec![];
    for i in 100..110 {
        token_ids.push(i.to_string());
    }

    // convert
    let res = owner
        .call(&worker, escrow_contract.id(), "convert")
        .args(json!({"token_ids": token_ids}).to_string().as_bytes().to_vec())
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    assert_eq!(owner
       .call(
           &worker,
           escrow_contract.id(),
           "pt_balance_of",
       )
       .args_json((owner.id(), vec![100.to_string(), 101.to_string()]))?
       .view()
       .await?
       .json::<Vec<u128>>()?, vec![0u128.into(), 0u128.into()]);

    Ok(())
}


#[tokio::test]
async fn test_revert_ft_convert() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (escrow_contract, stable_coin_contract, owner, _, _, finder, _, _) = init(&worker).await?;

    let res = owner
        .call(&worker, escrow_contract.id(), "active_ft_project".into())
        .args_json((FT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    // println!("active: {:?}", res);

    //buy proxy token
    //calculate stable coin amount for buying proxy token
    let amount = U128::from(100u128);
    let coin_amount = escrow_contract
        .view(
            &worker,
            "calculate_buy_proxy_token",
            json!({
            "amount": amount
        }).to_string().into_bytes(),
        )
        .await?
        .json::<u128>()?;

    let _res = owner
        .call(&worker, stable_coin_contract.id(), "ft_transfer_call".into())
        .args_json((escrow_contract.id(), U128(coin_amount), Option::<String>::None, format!("buy:{}", amount.0)))?
        .deposit(1u128)
        .max_gas()
        .transact()
        .await?;

    worker.fast_forward(300).await?;

    let mut token_ids: Vec<String> = vec![];
    for i in 0..49 {
        token_ids.push(i.to_string());
    }

    // convert
    let res = owner
        .call(&worker, escrow_contract.id(), "convert")
        .args(json!({"token_ids": token_ids}).to_string().as_bytes().to_vec())
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    println!("convert: {:?}", res);

    // not burned proxy token
    assert_eq!(owner
       .call(&worker, escrow_contract.id(), "pt_balance_of", )
       .args_json((owner.id(), vec![0.to_string(), 1.to_string()]))?
       .view()
       .await?
       .json::<Vec<u128>>()?, vec![1u128.into(), 1u128.into()]);
    Ok(())
}


#[tokio::test]
async fn test_ft_convert() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (escrow_contract, stable_coin_contract, owner, _, _, finder, _, _) = init(&worker).await?;

    owner
        .call(&worker, escrow_contract.id(), "active_ft_project".into())
        .args_json((FT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;

    //buy proxy token
    //calculate stable coin amount for buying proxy token
    let amount = U128::from(100u128);
    let coin_amount = escrow_contract
        .view(
            &worker,
            "calculate_buy_proxy_token",
            json!({
            "amount": amount
        }).to_string().into_bytes(),
        )
        .await?
        .json::<u128>()?;

    let _res = owner
        .call(&worker, stable_coin_contract.id(), "ft_transfer_call".into())
        .args_json((escrow_contract.id(), U128(coin_amount), Option::<String>::None, format!("buy:{}", amount.0)))?
        .deposit(1u128)
        .max_gas()
        .transact()
        .await?;

    worker.fast_forward(300).await?;

    // register account
    let project_token_id = escrow_contract.call(&worker, "get_project_token_id")
        .view()
        .await?
        .json::<AccountId>()?;
    owner
        .call(&worker, &project_token_id, "storage_deposit")
        .args_json((owner.id(), Option::<bool>::None))?
        .deposit(125 * STORAGE_BYTE_COST)
        .max_gas()
        .transact()
        .await?;

    let mut token_ids: Vec<String> = vec![];
    for i in 0..49 {
        token_ids.push(i.to_string());
    }

    // convert
    owner
        .call(&worker, escrow_contract.id(), "convert")
        .args(json!({"token_ids": token_ids}).to_string().as_bytes().to_vec())
        .max_gas()
        .transact()
        .await?;

    // burned proxy token
    assert_eq!(owner
       .call(&worker, escrow_contract.id(), "pt_balance_of", )
       .args_json((owner.id(), vec![0.to_string(), 1.to_string()]))?
       .view()
       .await?
       .json::<Vec<u128>>()?, vec![0u128.into(), 0u128.into()]);

    Ok(())
}

#[tokio::test]
async fn test_claim_fund() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (escrow_contract, stable_coin_contract, owner, _, _, finder, _, _) = init(&worker).await?;

    owner
        .call(&worker, escrow_contract.id(), "active_ft_project".into())
        .args_json((FT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;

    //buy proxy token
    //calculate stable coin amount for buying proxy token
    let amount = U128::from(3u128);
    let coin_amount = escrow_contract
        .view(
            &worker,
            "calculate_buy_proxy_token",
            json!({
            "amount": amount
        }).to_string().into_bytes(),
        )
        .await?
        .json::<u128>()?;

    let _res = owner
        .call(&worker, stable_coin_contract.id(), "ft_transfer_call".into())
        .args_json((escrow_contract.id(), U128(coin_amount), Option::<String>::None, format!("buy:{}", amount.0)))?
        .deposit(1u128)
        .max_gas()
        .transact()
        .await?;

    // pass buffer period
    worker.fast_forward(400).await?;
 
    // register account
    let project_token_id = escrow_contract.call(&worker, "get_project_token_id")
        .view()
        .await?
        .json::<AccountId>()?;
    owner
        .call(&worker, &project_token_id, "storage_deposit")
        .args_json((owner.id(), Option::<bool>::None))?
        .deposit(125 * STORAGE_BYTE_COST)
        .max_gas()
        .transact()
        .await?;

    // convert
    let res = owner
        .call(&worker, escrow_contract.id(), "convert")
        .args(json!({"token_ids": vec![2.to_string(), 3.to_string()]}).to_string().as_bytes().to_vec())
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    // pass conversion period
    worker.fast_forward(800).await?;

    let total_fund_amount = 
        escrow_contract.call(&worker, "get_total_fund_amount")
            .view()
            .await?
            .json::<u128>()?;
    let total_finder_fee = total_fund_amount * FINDER_FEE as u128 / 100u128;

    // claim fund
    let res = owner
        .call(&worker, escrow_contract.id(), "claim_fund")
        .args(json!({"to": owner.id(), "amount": U128(total_fund_amount - total_finder_fee)}).to_string().as_bytes().to_vec())
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    // claim fund
    let res = owner
        .call(&worker, escrow_contract.id(), "claim_finder_fee")
        .args(json!({"amount": U128(total_finder_fee)}).to_string().as_bytes().to_vec())
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    let finder_balance = stable_coin_contract
        .view(
            &worker,
            "ft_balance_of",
            json!({
                "account_id": finder.id()
            }).to_string().into_bytes()
        )
        .await?
        .json::<U128>()?;

    assert_eq!(finder_balance.0, total_finder_fee);

    Ok(())
}

#[tokio::test]
async fn test_close_nft_project() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (escrow_contract, stable_coin_contract, owner, alice, _, finder, _, _) = init(&worker).await?;

    // pre-mint
    owner
        .call(&worker, escrow_contract.id(), "pre_mint")
        .args(json!({"amount": PRE_MINT_AMOUNT}).to_string().as_bytes().to_vec())
        .deposit(PRE_MINT_AMOUNT.0 * DEPOSIT_ONE_PT_MINT)
        .max_gas()
        .transact()
        .await?;

    // active project
    owner
        .call(&worker, escrow_contract.id(), "active_nft_project")
        .args_json((NFT_BASE_URI, NFT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;

    //buy proxy token
    //calculate stable coin amount for buying proxy token
    let amount = U128::from(100u128);
    let coin_amount = escrow_contract
        .view(
            &worker,
            "calculate_buy_proxy_token",
            json!({
            "amount": amount
        }).to_string().into_bytes(),
        )
        .await?
        .json::<u128>()?;

    alice
        .call(&worker, stable_coin_contract.id(), "ft_transfer_call".into())
        .args_json((escrow_contract.id(), U128(coin_amount), Option::<String>::None, format!("buy:{}", amount.0)))?
        .deposit(1u128)
        .max_gas()
        .transact() 
        .await?;

    worker.fast_forward(400).await?;
    
    // convert
    alice
        .call(&worker, escrow_contract.id(), "convert")
        .args(json!({"token_ids": vec![100.to_string()]}).to_string().as_bytes().to_vec())
        .max_gas()
        .transact()
        .await?;

    worker.fast_forward(800).await?;

    // close project - pre-mint
    let res = owner
        .call(&worker, escrow_contract.id(), "close_project")
        .args_json(json!({}).to_string().as_bytes().to_vec())?
        .max_gas()
        .transact()
        .await?;
    println!("closed1: {:?}", res);

    // // close project - mint remaining proxys
    // let res = owner
    //     .call(&worker, escrow_contract.id(), "close_project")
    //     .args_json(json!({}).to_string().as_bytes().to_vec())?
    //     .max_gas()
    //     .transact()
    //     .await?;
    // println!("closed2: {:?}", res);
    //
    // // close project - transfer ownership
    // let res = owner
    //     .call(&worker, escrow_contract.id(), "close_project")
    //     .args_json(json!({}).to_string().as_bytes().to_vec())?
    //     .max_gas()
    //     .transact()
    //     .await?;
    // println!("closed3: {:?}", res);

    // println!("close project: {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_convert_after_closing_nft_project() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (escrow_contract, stable_coin_contract, owner, alice, _, finder, _, _) = init(&worker).await?;

    // active project
    owner
        .call(&worker, escrow_contract.id(), "active_nft_project")
        .args_json((NFT_BASE_URI, NFT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
        .max_gas()
        .transact()
        .await?;

    //buy proxy token
    //calculate stable coin amount for buying proxy token
    let amount = U128::from(10u128);
    let coin_amount = escrow_contract
        .view(
            &worker,
            "calculate_buy_proxy_token",
            json!({
            "amount": amount
        }).to_string().into_bytes(),
        )
        .await?
        .json::<u128>()?;

    alice
        .call(&worker, stable_coin_contract.id(), "ft_transfer_call".into())
        .args_json((escrow_contract.id(), U128(coin_amount), Option::<String>::None, format!("buy:{}", amount.0)))?
        .deposit(1u128)
        .max_gas()
        .transact() 
        .await?;

    worker.fast_forward(400).await?;
    
    // convert
    let res = alice
        .call(&worker, escrow_contract.id(), "convert")
        .args(json!({"token_ids": vec![2.to_string()]}).to_string().as_bytes().to_vec())
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());
    println!("convert: {:?}", res);

    worker.fast_forward(800).await?;

    // close project
    owner
        .call(&worker, escrow_contract.id(), "close_project")
        .args_json(json!({}).to_string().as_bytes().to_vec())?
        .max_gas()
        .transact()
        .await?;

    owner
        .call(&worker, escrow_contract.id(), "close_project")
        .args_json(json!({}).to_string().as_bytes().to_vec())?
        .max_gas()
        .transact()
        .await?;

    owner
        .call(&worker, escrow_contract.id(), "close_project")
        .args_json(json!({}).to_string().as_bytes().to_vec())?
        .max_gas()
        .transact()
        .await?;

    // println!("close project!");

    let res = alice
        .call(&worker, escrow_contract.id(), "convert")
        .args(json!({"token_ids": vec![3.to_string(),4.to_string(),5.to_string()]}).to_string().as_bytes().to_vec())
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    // println!("convert: {:?}", res);

    Ok(())
}


// #[tokio::test]
// async fn test_convert_after_closing_ft_project() -> anyhow::Result<()> {
//     let worker = workspaces::sandbox().await?;
//     let (escrow_contract, stable_coin_contract, owner, alice, _, finder, _, _) = init(&worker).await?;

//     // active project
//     let res = owner
//         .call(&worker, escrow_contract.id(), "active_ft_project".into())
//         .args_json((FT_MAX_SUPPLY, finder.id(), FUND_THRESHOLD, FIVE_MINUTES, TEN_MINUTES))?
//         .max_gas()
//         .transact()
//         .await?;
//     assert!(res.is_success());
//     // println!("active: {:?}", res);

//     //buy proxy token
//     //calculate stable coin amount for buying proxy token
//     let amount = U128::from(3u128);
//     let coin_amount = escrow_contract
//         .view(
//             &worker,
//             "calculate_buy_proxy_token",
//             json!({
//             "amount": amount
//         }).to_string().into_bytes(),
//         )
//         .await?
//         .json::<u128>()?;

//     let _res = owner
//         .call(&worker, stable_coin_contract.id(), "ft_transfer_call".into())
//         .args_json((escrow_contract.id(), U128(coin_amount), Option::<String>::None, format!("buy:{}", amount.0)))?
//         .deposit(1u128)
//         .max_gas()
//         .transact()
//         .await?;

//     worker.fast_forward(300).await?;

//     // register account
//     let project_token_id = escrow_contract.call(&worker, "get_project_token_id")
//         .view()
//         .await?
//         .json::<AccountId>()?;
//     owner
//         .call(&worker, &project_token_id, "storage_deposit")
//         .args_json((owner.id(), Option::<bool>::None))?
//         .deposit(125 * STORAGE_BYTE_COST)
//         .max_gas()
//         .transact()
//         .await?;

//     // convert
//     let res = owner
//         .call(&worker, escrow_contract.id(), "convert")
//         .args(json!({"token_ids": vec![2.to_string(), 3.to_string()]}).to_string().as_bytes().to_vec())
//         .max_gas()
//         .transact()
//         .await?;
//     assert!(res.is_success());

//     worker.fast_forward(800).await?;

//     owner
//         .call(&worker, &project_token_id, "storage_deposit")
//         .args_json((escrow_contract.id(), Option::<bool>::None))?
//         .deposit(125 * STORAGE_BYTE_COST)
//         .max_gas()
//         .transact()
//         .await?;

//     // close project
//     let res = owner
//         .call(&worker, escrow_contract.id(), "close_project")
//         .args_json(json!({}).to_string().as_bytes().to_vec())?
//         .max_gas()
//         .transact()
//         .await?;
//     assert!(res.is_success());

//     // println!("close project!");

//     let res = alice
//         .call(&worker, escrow_contract.id(), "convert")
//         .args(json!({"token_ids": vec![3.to_string(),4.to_string(),5.to_string()]}).to_string().as_bytes().to_vec())
//         .max_gas()
//         .transact()
//         .await?;
//     assert!(res.is_success());

//     println!("convert: {:?}", res);

//     Ok(())
// }

