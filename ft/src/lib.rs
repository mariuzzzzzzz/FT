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

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3C%3Fxml%20version%3D%221.0%22%20encoding%3D%22UTF-8%22%20standalone%3D%22no%22%3F%3E%3Csvg%20xml%3Aspace%3D%22preserve%22%20viewBox%3D%220%200%20562%20562%22%20version%3D%221.1%22%20id%3D%22svg21%22%20%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%3E%20%20%3Cpath%20fill%3D%22%2300D8E9%22%20d%3D%22m330%20494-5%202-16%203c-20%206-42%204-63%204l-30-2c-5%200-10%200-14-2-6-4-14-4-20-7-3-2-7-1-9-3-5-6-12-4-17-10-4-4-12-6-18-9l-7-5-10-6-9-9-10-9-7-8c-5-6-11-11-14-18-4-7-10-12-15-18l-7-16-2-4c-4-4-4-10-6-15-2-6-6-12-5-19-6-3-3-10-5-15-3-4-2-10-2-16l-1-56%202-20c1-4%201-10%203-13%203-5%202-9%203-13%202-4%206-8%206-12%200-9%207-14%2010-22%203-10%2010-19%2016-27l12-15%208-8%2014-14%2011-8c3-4%209-4%2011-9l3-2%2017-8%2017-10%2015-5c5-3%2011-1%2015-5%202-2%205-2%207-2%2026-4%2052-3%2077-3%2011%200%2022%202%2033%204%205%201%209%204%2013%206l14%204%2020%209%2020%2011c4%202%206%206%209%208l3%202h3l14%2015%207%207%2010%209%208%2011%208%2012c3%204%208%208%208%2014l5%207%208%2018%206%2013%201%207c3%2011%206%2022%206%2034v19c0%2020%202%2041-4%2060l-6%2025c-3%2012-9%2023-15%2033-5%209-9%2020-18%2027-5%207-10%2015-17%2021l-19%2018-15%2011-26%2016-20%209-11%204m38-294c1-3-2-7%203-9%202-1%203-4%204-6%203-10%203-19-5-27-6-5-13-10-22-10-4%200-5%202-7%203-10%203-13%2011-17%2019H208c0-8%201-7-8-19-3-4-7-6-11-6-8%200-17-2-24%206-4%205-7%2011-9%2017-2%204%201%209%203%2013%201%202%202%205%204%206%205%204%206%208%204%2014-1%204-1%209-5%2013-2%203-1%209-2%2013%200%202%200%205-2%207-3%204-4%209-5%2014l-8%2024-2%204-9%204c-5%203-9%207-10%2013-2%209-3%2018%206%2026%205%205%2010%209%2017%209%206%200%2012%200%2016%205l5%202%2016%2012%2017%2010%2015%2012%2014%208%2014%2010%209%206c-1%2010%205%2017%2014%2023%207%204%2013%201%2020%201%202%200%205-1%207-3l5-6c3-2%203-5%204-8%201-2%204-13%202-14-5-2-2-6-3-8l5-4c6-4%2012-7%2015-12%204-5%2010-7%2014-12%207-8%2017-14%2025-21l3-1c7-1%2014%201%2021-4%205-4%209-7%2011-13l1-13c0-7-6-15-12-18-3-2-6-4-7-7l-2-21c-1-9-2-18-5-27-2-8-2-16-3-25z%22%20id%3D%22path11%22%20%2F%3E%20%20%3Cpath%20fill%3D%22%23041858%22%20d%3D%22m330%20494%2011-4%2020-9%2026-16%2015-11%2019-18c7-6%2012-14%2017-21%209-7%2013-18%2018-27%206-10%2012-21%2015-33l6-25c6-19%204-40%204-60v-19l10%2014%2012%2015%2012%2015%207%208c2%202%205%204%205%208l5%203c4%209%203%2015-7%2022l-19%2013c-4%203-7%207-9%2011-1%203%200%207%201%2010%201%204%206%207%205%2010%200%206-4%2010-9%2013l-8%206%202%201c4%200%209%200%209%205%201%205%201%2010-3%2014-6%206-12%2012-10%2021l5%2017%201%207c1%209-2%2017-9%2023l-7%202c-12%205-24%204-36%202-22-3-44-2-66-1l-40%205-2-1z%22%20id%3D%22path13%22%20%2F%3E%20%20%3Cpath%20d%3D%22m368%20201%203%2024c3%209%204%2018%205%2027l2%2021c1%203%204%205%207%207%206%203%2012%2011%2012%2018l-1%2013c-2%206-6%209-11%2013-7%205-14%203-21%204l-3%201c-8%207-18%2013-25%2021-4%205-10%207-14%2012-3%205-9%208-15%2012l-5%204c1%202-2%206%203%208%202%201-1%2012-2%2014-1%203-1%206-4%208l-5%206c-2%202-5%203-7%203-7%200-13%203-20-1-9-6-15-13-14-23l-9-6-14-10-14-8-15-12-17-10-16-12-5-2c-4-5-10-5-16-5-7%200-12-4-17-9-9-8-8-17-6-26%201-6%205-10%2010-13l9-4%202-4%208-24c1-5%202-10%205-14%202-2%202-5%202-7%201-4%200-10%202-13%204-4%204-9%205-13%202-6%201-10-4-14-2-1-3-4-4-6-2-4-5-9-3-13%202-6%205-12%209-17%207-8%2016-6%2024-6%204%200%208%202%2011%206%209%2012%208%2011%208%2019h116c4-8%207-16%2017-19%202-1%203-3%207-3%209%200%2016%205%2022%2010%208%208%208%2017%205%2027-1%202-2%205-4%206-5%202-2%206-3%2010m-46-21H208c-3%203-4%208-9%209l1%204%205%208c2%205%202%209%206%2013%203%204%203%2010%205%2015l5%208%206%2015c3%206%207%207%2012%208l14%204c1-3%202-5%204-6l12-8c6-6%2011-12%2019-15l1-1%206-7%2014-13%2015-10%207-7-1-5-3-5-5-7m-60%20104%2033%203c12%202%2024%200%2036%203h18c4-11%2014-11%2022-16v-7l-4-14-1-15-1-10-4-14-1-13h-13c-4%200-7-1-10%203l-9%208c-2%203-4%205-9%205%201%207-6%207-9%2010l-10%208-10%209-17%2014-13%2010-1%204%203%2012m-2%2011c-1%204%200%209-8%2011l4%209%202%206%204%2010%203%204%206%2018%203%205%205%208c3%200%208%200%2012%205%201%201%205%201%207-1%205-3%208-9%2015-11v-1l10-10c7-5%2015-10%2020-17l3-2c4-2%207-6%207-9-6-6-7-13-10-18l-28-4c-11-2-23%203-35-3-5-2-13-1-20%200m-19%2019c-7%201-13-1-18-5-4-2-6-6-9-8-1-2-3-3-5-3l-16%202-16%201-6%2015-2%202c-4%201-2%204-2%205l6%206c11%204%2020%2011%2029%2018l8%205%2015%2010%2015%2010%207%206c3%201%207%201%209-2l7-5c6-1%204-5%203-8%200-3-2-6-4-10l-7-14-1-5c-2-4-5-8-6-13-1-3-4-5-7-7m-73-84c-3%2010-4%2021-9%2030l-1%203-4%2013c6%204%2014%205%2016%2013l3%202h7l15-3%2013-1c3-6%204-12%208-15%202-3%204-6%204-9%200-4-2-8-5-12l-11-27c-1-3-1-6-3-8-4-3-4-7-5-11-1-3-4-6-7-8h-11l-1%206-4%2010-3%2015-2%202z%22%20id%3D%22path15%22%20%2F%3E%20%20%3Cpath%20fill%3D%22%2300D8E9%22%20d%3D%22m323%20180%204%207%203%205%201%205-7%207-15%2010-14%2013-6%207-1%201c-8%203-13%209-19%2015l-12%208c-2%201-3%203-4%206l-14-4c-5-1-9-2-12-8l-6-15-5-8c-2-5-2-11-5-15-4-4-4-8-6-13l-5-8-1-4c5-1%206-6%2010-9l4%201h106l4-1zM262%20284l-3-12%201-4%2013-10%2017-14%2010-9%2010-8c3-3%2010-3%209-10%205%200%207-2%209-5l9-8c3-4%206-3%2010-3h13l1%2013%204%2014%201%2010%201%2015%204%2014v7c-8%205-18%205-22%2016h-18c-12-3-24-1-36-3l-33-3zM261%20294c6%200%2014-1%2019%201%2012%206%2024%201%2035%203l28%204c3%205%204%2012%2010%2018%200%203-3%207-7%209l-3%202c-5%207-13%2012-20%2017l-10%2010v1c-7%202-10%208-15%2011-2%202-6%202-7%201-4-5-9-5-12-5l-5-8-3-5-6-18-3-4-4-10-2-6-4-9c8-2%207-7%209-12zM242%20314c2%202%205%204%206%207%201%205%204%209%206%2013l1%205%207%2014c2%204%204%207%204%2010%201%203%203%207-3%208l-7%205c-2%203-6%203-9%202l-7-6-15-10-15-10-8-5c-9-7-18-14-29-18l-6-6c0-1-2-4%202-5l2-2%206-15%2016-1%2016-2c2%200%204%201%205%203%203%202%205%206%209%208%205%204%2011%206%2019%205zM168%20230l2-2%203-15%204-10%201-6h11c3%202%206%205%207%208%201%204%201%208%205%2011%202%202%202%205%203%208l11%2027c3%204%205%208%205%2012%200%203-2%206-4%209-4%203-5%209-8%2015l-13%201-15%203h-7l-3-2c-2-8-10-9-16-13l4-13%201-3c5-9%206-20%209-30z%22%20id%3D%22path17%22%20%2F%3E%20%20%3Cpath%20fill%3D%22%2300D5D5%22%20d%3D%22m323%20180-4%201H213l-4-1h114z%22%20id%3D%22path19%22%20%2F%3E%3C%2Fsvg%3E";

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
