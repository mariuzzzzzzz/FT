/*!
Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId, total_supply: U128) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
        )
    }

    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(
        owner_id: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        let icon = Some("data:image/svg+xml,%3Csvg%20version%3D%221.1%22%20id%3D%22layer%22%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20xmlns%3Axlink%3D%22http%3A%2F%2Fwww.w3.org%2F1999%2Fxlink%22%20x%3D%220px%22%20y%3D%220px%22%20viewBox%3D%220%200%20652%20652%22%20style%3D%22enable-background%3Anew%200%200%20652%20652%3B%22%20xml%3Aspace%3D%22preserve%22%3E%3Cstyle%20type%3D%22text%2Fcss%22%3E.st0%7Bfill%3A%230064A6%3B%7D%3C%2Fstyle%3E%3Cg%3E%3Cpath%20class%3D%22st0%22%20d%3D%22M552.5%2C375.1c-5.6-6.4-13.7-10-22.1-10.1C518%2C365.1%2C507%2C373%2C503%2C384.7c0%2C0-1%2C3.6-1%2C3.6l-1.2%2C5.3c0%2C0-19.9%2C88.6-23.1%2C102.6c-0.8-3.7-19.9-92-19.9-92c-0.8-3.8-2-8.7-3.5-13.2l-1.3-3.8c-5.6-13.4-18.5-22-33-22c-13.8%2C0-26.5%2C8.1-32.4%2C20.7c-1%2C2.6-1.9%2C5.6-2.8%2C9c-0.8%2C3.2-1.5%2C6.5-2.1%2C9.3c0%2C0-16.9%2C79.3-19.7%2C92.4c-0.8-3.6-24.8-103-24.8-103l-1-4.2c-0.7-2.6-1.3-4.7-1.9-6.5c-4.6-10.7-15.1-17.6-26.7-17.7c-8.4%2C0-16.5%2C3.7-22.1%2C10.1c-4.1%2C4.7-6.5%2C10.5-7%2C16.8l-0.1%2C1.2c0%2C1.7%2C0.1%2C3.4%2C0.3%2C5.2c0.6%2C6%2C2.1%2C10.8%2C3.5%2C15.5l38.8%2C126.4l0.8%2C2.7l2%2C5.8c6%2C13.7%2C19.5%2C22.6%2C34.4%2C22.7h0.3c15.5%2C0%2C29.2-9.4%2C34.9-23.8c0-0.1%2C1.8-6.4%2C1.8-6.4c0%2C0%2C0.3-1.2%2C0.3-1.2s22.1-88%2C22.9-91.1c0.8%2C3.2%2C23.2%2C92.3%2C23.2%2C92.3l1.7%2C5.9c5.5%2C14.5%2C19.6%2C24.3%2C35.1%2C24.3h0.3c14.6-0.1%2C28-8.7%2C34.1-22c0.8-2.1%2C1.6-4.4%2C2.4-7l0.6-2.1l38.9-126.4c1.4-4.7%2C3-9.5%2C3.5-15.5c0.2-1.8%2C0.2-3.4%2C0.2-5.2l0-1.2C559.1%2C385.7%2C556.6%2C379.9%2C552.5%2C375.1%20M488.8%2C142.9c-22.4%2C0-41.7%2C7.8-56%2C22.4v-63.6v-0.5c-0.3-16.5-14-29.9-30.4-29.9c-16.8%2C0-30.4%2C13.7-30.4%2C30.4v217.9c0%2C16.8%2C13.6%2C30.4%2C30.4%2C30.4c16.6%2C0%2C30.2-13.5%2C30.4-30v-0.4v-87.5c0-23.4%2C18.9-37.9%2C36.4-37.9c25.7%2C0%2C29.5%2C20.5%2C29.5%2C32.8v92.6l0%2C0.5c0.2%2C16.5%2C13.9%2C29.9%2C30.4%2C29.9c16.8%2C0%2C30.4-13.7%2C30.4-30.4V216.6C559.6%2C170.5%2C533.1%2C142.9%2C488.8%2C142.9%20M269.6%2C512v-89c0-37.1-31.7-57.5-89.1-57.5c-53.9%2C0-87.2%2C25.6-87.2%2C50.1c0%2C13.6%2C11%2C24.6%2C24.6%2C24.6c5.9%2C0%2C11.3-2.1%2C15.6-5.6c2.7-2%2C5.2-4.4%2C7.7-6.9c8.2-8%2C17.5-17%2C38.9-17c14.7%2C0%2C30.4%2C4.9%2C30.4%2C18.8c0%2C12.1-6.8%2C15.1-20.2%2C16.6l-32.3%2C3.6c-38%2C4.4-76.8%2C15.5-76.8%2C64c0%2C35.8%2C32%2C57.1%2C62.9%2C57.1c29.1%2C0%2C51.6-8.8%2C70.4-27.3c2.9%2C17.6%2C14.6%2C27.3%2C33.5%2C27.3c15.1%2C0%2C27-10.3%2C27-23.4c0-3.6-0.9-6.6-1.9-10.1C271.6%2C532.1%2C269.6%2C525.3%2C269.6%2C512%20M213.5%2C494.6c0%2C19.6-17.8%2C38.5-46.8%2C38.5c-18.2%2C0-29.4-9.8-29.4-21.8c0-16%2C12-22.9%2C34.1-26.2l19.3-2.9c6.2-1.1%2C17.1-2.9%2C22.9-8.4V494.6z%20M180.4%2C176.9c0%2C10.7%2C6.8%2C20.1%2C17%2C23.5c2.7%2C0.8%2C5.5%2C1.1%2C8.3%2C1.1h67.8c-2.6%2C2.8-93.9%2C102.1-93.9%2C102.1c-5.3%2C5.6-7.9%2C13-7.9%2C21.9l0%2C1.2c0.5%2C10.8%2C8.8%2C22.7%2C25.8%2C23.4l1.7%2C0.1h135.1c2.9%2C0%2C5.7-0.4%2C8.4-1.2c10.1-3.4%2C16.9-12.7%2C16.9-23.4c0-10.6-6.8-20.1-16.9-23.4v0c-2.7-0.8-5.5-1.2-8.4-1.2H256c2.6-2.8%2C92.1-101%2C92.1-101c6-6.4%2C9.4-13.8%2C9.4-20.4l-0.1-2.5c-0.3-4.5-1.3-8.5-3.1-11.9c-2.8-5.5-7.4-9.4-13.3-11.3c-2.8-0.9-6-1.4-9.2-1.4h-126c-2.9%2C0-5.6%2C0.4-8.3%2C1.1C187.2%2C156.9%2C180.4%2C166.3%2C180.4%2C176.9%22%20%2F%3E%3C%2Fg%3E%3C%2Fsvg%3E".to_string());
        let metadata = FungibleTokenMetadata { spec: metadata.spec, name: metadata.name, symbol: metadata.symbol, icon, reference:None, reference_hash: None, decimals: metadata.decimals };
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
        };
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());
        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &owner_id,
            amount: &total_supply,
            memo: Some("Initial tokens supply is minted"),
        }
        .emit();
        this
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}
