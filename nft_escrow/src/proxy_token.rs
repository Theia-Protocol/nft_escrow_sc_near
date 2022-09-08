use near_contract_standards::non_fungible_token::{refund_deposit_to_account};
use crate::*;

#[near_bindgen]
impl Contract {
    /// Mint nft tokens with amount belonging to `receiver_id`.
    /// caller should be owner
    #[payable]
    pub fn pt_mint(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
    ) -> Vec<TokenId> {
        self.assert_owner();
        assert!(amount.0 > 0, "Invalid amount");
        assert!(self.pt_all_total_supply + amount.0 < self.pt_max_supply, "OverMaxSupply");

        let mut token_ids: Vec<TokenId> = vec![];
        let mut i = 0;

        // Remember current storage usage if refund_id is Some
        let refund_id = Some(env::predecessor_account_id());
        let initial_storage_usage = refund_id.as_ref().map(|account_id| (account_id, env::storage_usage()));

        while i < amount.0 {
            let token_id: TokenId = self.pt_all_total_supply.checked_add(i).unwrap().to_string();

            // Insert new supply
            self.pt_total_supply.insert(
                &token_id,
                &self
                    .pt_total_supply
                    .get(&token_id).unwrap_or(0)
                    .checked_add(1)
                    .unwrap_or_else(|| env::panic_str("Total supply overflow")));

            // Insert new balance
            if self.pt_balances_per_token.get(&token_id).is_none() {
                let mut new_set: LookupMap<AccountId, u128> = LookupMap::new(StorageKey::BalancesInner {
                    token_id: env::sha256(token_id.as_bytes()),
                });
                new_set.insert(&receiver_id, &1u128);
                self.pt_balances_per_token.insert(&token_id, &new_set);
            } else {
                let new = self.pt_balances_per_token.get(&token_id).unwrap().get(&receiver_id).unwrap_or(0).checked_add(1).unwrap();
                let mut balances = self.pt_balances_per_token.get(&token_id).unwrap();
                balances.insert(&receiver_id, &new);
            }

            token_ids.push(token_id.clone());
            i += 1;

            MtMint {
                owner_id: &receiver_id,
                token_ids: &[&token_id],
                amounts: &["1"],
                memo: None,
            }
                .emit();
        }

        if let Some((id, usage)) = initial_storage_usage {
            refund_deposit_to_account(env::storage_usage() - usage, id.clone());
        }

        self.pt_all_total_supply = self.pt_all_total_supply.checked_add(amount.0).unwrap();

        token_ids
    }

    /// Burn nft tokens from `from_id`.
    /// caller should be owner
    pub fn pt_burn(
        &mut self,
        from_id: AccountId,
        token_ids: Vec<TokenId>,
    ) -> bool {
        assert!(token_ids.len() > 0, "Invalid param");
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Unauthorized");

        token_ids.iter().enumerate().for_each(|(_, token_id)| {
            let balance = self.internal_unwrap_balance_of(token_id, &from_id);
            if let Some(new) = balance.checked_sub(1) {
                let mut balances = self.pt_balances_per_token.get(token_id).unwrap();
                balances.insert(&from_id, &new);
                self.pt_total_supply.insert(
                    token_id,
                    &self
                        .pt_total_supply
                        .get(token_id)
                        .unwrap()
                        .checked_sub(1)
                        .unwrap_or_else(|| env::panic_str("Total supply overflow")),
                );
            } else {
                env::panic_str("The account doesn't have enough balance");
            }

            MtBurn {
                owner_id: &from_id,
                authorized_id: Some(&self.owner_id),
                token_ids: &[&token_id],
                amounts: &["1"],
                memo: None,
            }
                .emit();
        });

        self.pt_all_total_supply = self.pt_all_total_supply.checked_sub(token_ids.len().try_into().unwrap()).unwrap();

        true
    }

    pub fn pt_token(&self, token_id: TokenId) -> Option<Token> {
        let metadata = ProxyTokenMetadata {
            title: Some(token_id.clone()),
            description: None,
            media: Some(self.pt_media_uri.clone()),
            media_hash: None,
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None
        };
        let supply = self.pt_total_supply.get(&token_id)?;

        Some(Token {
            token_id,
            owner_id: None,
            supply,
            metadata
        })
    }

    /// Used to get balance of specified account in specified token
    pub fn internal_unwrap_balance_of(
        &self,
        token_id: &TokenId,
        account_id: &AccountId,
    ) -> Balance {
        match self
            .pt_balances_per_token
            .get(token_id)
            .expect("This token does not exist")
            .get(account_id)
        {
            Some(balance) => balance,
            None => {
                env::panic_str(format!("The account {} is not registered", account_id).as_str())
            }
        }
    }

    pub fn pt_balance_of(&self, owner: AccountId, id: Vec<TokenId>) -> Vec<u128> {
        self.pt_balances_per_token
            .iter()
            .filter(|(token_id, _)| id.contains(token_id))
            .map(|(_, balances)| {
                balances
                    .get(&owner)
                    .expect("User does not have account in of the tokens")
            })
            .collect()
    }

    pub fn pt_metadata(&self) -> PTContractMetadata {
        return PTContractMetadata {
            spec: PT_METADATA_SPEC.to_string(),
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            icon: None,
            base_uri: None,
            reference: None,
            reference_hash: None,
        };
    }

    pub fn pt_pt_all_total_supply(&self) -> Balance { self.pt_all_total_supply.clone() }
}