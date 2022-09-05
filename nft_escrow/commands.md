
## Call Functions  
- 1. buy

    ```
    near call <stable-coin-id> ft_transfer_call '{"receiver_id":"<escrow-contract-id>","amount":"30000000000000000000000000","memo":"","msg":"buy:10"}' --accountId <user-account-id> --deposit 0.000000000000000000000001 --gas 300000000000000
    ```

- 2. sell

    ```
    near call <escrow-contract-id> sell '{"token_ids":["15","16","17","18","19"]}' --accountId <user-account-id> --gas 300000000000000
    ```

- 3. convert (register user account to project token)

    ```
    near call <escrow-contract-id> convert '{"token_ids":["25","26","27","28","29","30","31","32","33","34"]}' --accountId <user-account-id> --gas 300000000000000
    ```

- 4. claim fund (register project owner to stable coin)

    ```
    near call <escrow-contract-id> claim_fund '{"to":"<owner-account-id>","amount":"41880000000000000000000020"}' --accountId <owner-account-id> --gas 300000000000000
    ```

- 5. close project (register project owner and contract to project token)

    ```
    near call <escrow-contract-id> close_project '{}' --accountId <owner-account-id> --gas 300000000000000
    ```


## View Functions

- 1. get proxy token
    ```
    near view <escrow-contract-id> get_proxy_token_id
    ```

- 2. get project token
    ```
    near view <escrow-contract-id> get_project_token_id
    ```

- 3. calculate for buying proxy token
    ```
    near view <escrow-contract-id> calculate_buy_proxy_token '{"amount":"10"}'
    ```
    
- 4. calculate for selling proxy token
    ```
    near view <escrow-contract-id> calculate_sell_proxy_token '{"token_ids":["11","12","13","14","15"]}'
    ```

- 5. get proxy token Balance
    ```
    near view ptheiacollection1.<escrow-contract-id> mt_balance_of '{"owner":"<user-account-id>","id":["15","16","17","18","19"]}'
    ```

- 6. register account to project token (fungible token)
    ```
    near call theiacollection1.<escrow-contract-id> storage_deposit '{"account_id":"<user-account-id>"}' --deposit 0.01 --accountId <user-account-id>
    ```

- 7.  get total fund amount
    ```
    near view <escrow-contract-id> get_total_fund_amount
    ```

- 8.  get pre-mint amount
    ```
    near view <escrow-contract-id> get_pre_mint_amount
    ```

- 9.  get start timestamp
    ```
    near view <escrow-contract-id> get_start_timestamp
    ```

- 10.  get tp timestamp
    ```
    near view <escrow-contract-id> get_tp_timestamp
    ```

- 11.  get buffer period
    ```
    near view <escrow-contract-id> get_buffer_period
    ```

- 12.  get conversion period
    ```
    near view <escrow-contract-id> get_conversion_period
    ```

- 13.  get get_stable_coin_id
    ```
    near view <escrow-contract-id> get_stable_coin_id
    ```

- 14.  get running state
    ```
    near view <escrow-contract-id> get_running_state
    ```

- 15.  get `is_closed` flag
    ```
    near view <escrow-contract-id> get_is_closed
    ```

- 16.  get total converted amount
    ```
    near view <escrow-contract-id> get_converted_amount
    ```

- 17.  get current circulating supply amount
    ```
    near view <escrow-contract-id> get_circulating_supply
    ```
