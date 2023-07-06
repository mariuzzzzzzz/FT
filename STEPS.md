# Accounts

1 Create mainnet wallet using airdrop, store seed phrase somewhere safe

2 Create Github account: https://github.com/ 

# Development Environment

3 Fork repository https://github.com/near-examples/FT 

4 Open in Codespaces: https://github.com/codespaces 

5 Optional: change color theme to dark+

6 Change machine type to 4-core

7 Add dev container configuration for Rust: https://aka.ms/configure-codespace 

8 Restart codespace

9 Install rust nightly: rustup default nightly 

10 Install rust target: rustup target add wasm32-unknown-unknown 

# Build and deploy contract

11 Test contract:
   cd ft && cargo test -- --nocapture --color=always && cd ..

12 Build contract:
   scripts/build.sh

13 Fix incompatibility:
   tools/bin/wasm-opt -Oz --signext-lowering res/fungible_token.wasm -o res/fungible_token.wasm 

14 Install near-cli:
   npm install -g near-cli 

15 Switch to mainnet:
   export NEAR_ENV=mainnet

16 Login with near-cli:
   near-login 

17 Deploy contract:
   near deploy —wasmFile res/fungible_token.wasm —accountId $ID 

18 Look at transaction in explorer

19 Init contract:
   near call $ID new '{"owner_id": "'$ID'", "total_supply": "<TOTAL_TOKENS>", "metadata": { "spec": "ft-1.0.0", "name": "<YOUR_NAME>", "symbol": "<YOUR_SYMBOL>", "decimals": <DECIMALS> }}' --accountId $ID

20 Look at transaction in explorer

# Explore features

21 View token metadata:
   near view $ID ft_metadata 

22 Get an FT from https://nearblocks.io/tokens, look that metadata up

23 Get together in pairs

24 Register for your colleagues FT
   near call <COLLEAGUES_ACCOUNT> storage_deposit '' —accountId $ID —amount 0.00125 

25 Send your colleague some FT:
   near call $ID ft_transfer '{"receiver_id": "<COLLEAGUES_ACCOUNT>", "amount": "<AMOUNT_WITH_DECIMALS>"}' —accountId - $ID —amount 0.000000000000000000000001 

26 Check FT amounts at https://wallet.near.org 

# Explore FT and NFT standards

27 Look up NEPs, look at Nomicon

28 Find corresponding methods in our codebase

29 Modify methods, redeploy
