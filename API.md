#### deploy contract
```sh
export CONTRACT=#enter account_id here
#use path to .wasm file instead of `./res/near_lottery.wasm `
#deploy
near deploy --wasmFile $PATH --accountId $CONTRACT
#example
near deploy --wasmFile ./res/near_lottery.wasm --accountId example.testnet #testnet
near deploy --wasmFile ./res/near_lottery.wasm --accountId example.near #mainnet
```
#### initialize
```rust
///Config
/// - `contract_fee_ratio` need to be in range 0..10000 (0-100%)
/// - `treasury_ratio` need to be in range 0..10000 (0-100%)
/// - `investor_ratio` need to be in range 0..( 10000 - treasury_ratio ) (0-(100%-treasury_ratio))
/// example 
/// - 10% contract fees from winner reward  ( reward * 0.1 )
/// - 40% investor fees from contract fees  ( reward * 0.1 ) * 0.4
/// - 50% treasury fees from contract fees  ( reward * 0.1 ) * 0.5
/// will be 1000, 4000, 5000 for this setup
Config {
    /// contract owner
    pub owner_id: AccountId,
    /// fees taken from winner reward to contract in basis points (10%)
    pub contract_fee_ratio: u32,
    /// fees taken from `contract_fee` to treasury (0-60%)
    pub treasury_ratio: u32,
    /// fees taken from `contract_fee` to investor (40%)
    pub investor_ratio: u32,
    /// treasury account
    pub treasury: AccountId,
    /// investor account
    pub investor: AccountId,
    /// lotteries config
    pub lotteries_config: LotteryConfig
}

///Lottery config
/// - Required all lists (arrays) need to be non-empty
LotteryConfig {
    pub entry_fees: Vec<(<AccountId, Vec<U128>)>,
    pub num_participants: Vec<u32>,
    pub big_lottery_num_participants: Vec<u32>
}

/// - Panics if Config is not valid (see ratio requirements in Config description)
/// - Panics if LotteryConfig is not valid (see vec requirements in LotteryConfig description)
#[init]
pub fn new(
    config: Config,
    entry_fees: Vec<(AccountId, Vec<U128>)>,
    num_partcicipants: Vec<u32>,
    big_lottery_num_participants: Vec<u32>
) -> Contract
```
#### changeing a list of whitelisted tokens

```rust
/// Add FT to the whitelist.
/// - Requires one yoctoNEAR.
/// - Requires to be called by the contract owner.
/// - Requires this token not being already whitelisted.
#[payable]
pub fn whitelist_token(&mut self, token_id: AccountId)
/// Removes FT from the whitelist.
/// - Requires one yoctoNEAR.
/// - Requires to be called by the contract owner.
/// - Requires this token being already whitelisted.
#[payable]
pub fn remove_whitelist_token(&mut self, token_id: AccountId)
```

#### changeing entry fees & required num participants
- add new num of participants for lottery type
```rust
/// - Required at least 1 Yocto to attach
/// - Required to be called only from Owner's account
/// - Required lottery type from:
/// - SIMPLE_LOTTERY
/// - BIG_LOTTERY
#[payable]
pub fn add_num_participants(
    &mut self, 
    num: u32,
    lottery_type: String
)
```
- remove new num of participants for lottery type
```rust
/// - Required at least 1 Yocto to attach
/// - Required to be called only from Owner's account
/// - Required lottery type from:
///     - SIMPLE_LOTTERY
///     - BIG_LOTTERY
/// - Panics if this num was not set previously
#[payable]
pub fn remove_num_participants(
    &mut self, 
    num: u32,
    lottery_type: String
)

/// - Required at least 1 Yocto to attach
/// - Required to be called only from Owner's account
/// - If `token_id` was not set - add new fee instance for NEAR
/// - If some `token_id` given 
/// - Panics if token was not whitelisted before
/// - Panics if entry fee was already added
#[payable]
pub fn add_entry_fee(&mut self, token_id: Option<AccountId>, entry_fee: U128)

/// - Required at least 1 Yocto to attach
/// - Required to be called only from Owner's account
/// - If `token_id` was not set - remove this fee instance for NEAR
/// - If some `token_id` given 
/// - Panics if token was not whitelisted before
/// - Panics if entry fee was not already added (not found in `entry_fees[token_id] list`)
#[payable]
pub fn add_entry_fee(&mut self, token_id: Option<AccountId>, entry_fee: U128)
```

#### main API (enter a lottery)

- with NEAR
```rust
/// - Called from potential player account
/// - Required attached deposit equals to one from near entry fees ( E.g 1 or 2 or 5 NEAR)
/// - Required num_participants equals to one from lottery config num_partcicipants ( E.g 5,6,7,8,9,10 for SIMPLE_LOTTERY or 50 for BIG_LOTTERY )
/// - Required lottery type from:
///     - SIMPLE_LOTTERY
///     - BIG_LOTTERY
#[payable]
pub fn draw_near_enter(
    &mut self, 
    lottery_type: String,
    num_participants: u32,
    referrer_id: Option<AccountId>
) -> LotteryId 
```
- with any Fungible Token (FT)
```rust
/// - Called from fungible token account
/// - Panics if called from not whitelisted token account
/// - Required attached deposit equals to one from FT entry fees ( E.g 1 or 3 or 5 FT)
/// - Required num_participants equals to one from lottery config num_partcicipants ( E.g 5,6,7,8,9,10 for SIMPLE_LOTTERY or 50 for BIG_LOTTERY )
/// - Required lottery type from:
///     - SIMPLE_LOTTERY
///     - BIG_LOTTERY
/// - Referrer is optional. 
#[payable]
pub fn ft_transfer_call(
    sender_id: AccountId 
    amount: U128,
    // msg - DrawNearEnter    
    msg: String
) -> LotteryId 

DrawEnter {
    num_participants: u32,
    lottery_type: String,
    referrer_id: Option<AccountId>
}
/// E.g:
/// ```json
/// "msg": "{
///     \"DrawEnter\": {
///         \"num_participants\":5,         
///         \"lottery_type\":\"SIMPLE_LOTTERY\"
///     }
/// }"
/// ```
```