# LOTTERY

### * API will be here *

#### deploy
old one(NEAR ONLY) - 
```sh
near dev-deploy --wasmFile ./res/near_lottery_old.wasm
```

```sh
near dev-deploy --wasmFile ./res/near_lottery.wasm
```
#### env
```sh
#export CONTRACT=dev-1664736275925-68636536627316
export CONTRACT=dev-1669054085820-32539078279630
export OWNER=rmlsnk.testnet
export ONE_NEAR=1000000000000000000000000
export THREE_NEAR=3000000000000000000000000
export FIVE_NEAR=5000000000000000000000000
export TEN_NEAR=10000000000000000000000000
export ONE_USN=1000000000000000000

export USER_1=participant_1.sub.testnet
export USER_2=participant_2.sub.testnet
export USER_3=participant_3.sub.testnet
export USER_4=participant_4.sub.testnet
export USER_5=participant_5.sub.testnet
export GAS=300000000000000

export ACCEPTED_SUBS=sub.testnet
export INVESTOR=guacharo.testnet
export TREASURY=oilbird.testnet
```
#### method calls

```bash
near call $CONTRACT new '{
    "config": {
        "owner_id": "'$OWNER'",
        "contract_fee_ratio": 1000,
        "treasury_ratio": 0,
        "investor_ratio": 4000,
        "treasury": "'$TREASURY'",
        "investor": "'$INVESTOR'",
        "accepted_subs": "'$ACCEPTED_SUBS'"
    },
    "entry_fees": [
        ["near", [
            "'$ONE_NEAR'", 
            "'$THREE_NEAR'", 
            "'$FIVE_NEAR'"
        ]],
        ["usdn.testnet", [
            "'$ONE_USN'"
        ]]
    ],
    "num_participants": [
        5,6,7,8,9,10
    ],
    "big_lottery_num_participants":[
        50
    ]
}' --accountId $CONTRACT
```
- enter to lottery
#### WITH NEAR
```rust
pub fn draw_near_enter(
    &mut self, 
    //token_id: AccountId,
    lottery_type: String,
    num_participants: u32,
    entry_fee: U128
) -> LotteryId 
```
```sh
near call $CONTRACT draw_near_enter '{
    "lottery_type": "SIMPLE_LOTTERY",
    "num_participants": 5,
    "entry_fee": "'$ONE_NEAR'"
}' --accountId $USER_1 --depositYocto=$ONE_NEAR --gas=$GAS
near view $CONTRACT get_lottery '{
    "lottery_id": 1
}'

near call $CONTRACT draw_near_enter '{
    "lottery_type": "SIMPLE_LOTTERY",
    "num_participants": 5,
    "entry_fee": "'$ONE_NEAR'"
}' --accountId $USER_2 --depositYocto=$ONE_NEAR --gas=$GAS
near view $CONTRACT get_lottery '{
    "lottery_id": 1
}'

near call $CONTRACT draw_near_enter '{
    "lottery_type": "SIMPLE_LOTTERY",
    "num_participants": 5,
    "entry_fee": "'$ONE_NEAR'"
}' --accountId $USER_3 --depositYocto=$ONE_NEAR --gas=$GAS
near view $CONTRACT get_lottery '{
    "lottery_id": 1
}'

near call $CONTRACT draw_near_enter '{
    "lottery_type": "SIMPLE_LOTTERY",
    "num_participants": 5,
    "entry_fee": "'$ONE_NEAR'"
}' --accountId $USER_4 --depositYocto=$ONE_NEAR --gas=$GAS
near view $CONTRACT get_lottery '{
    "lottery_id": 1
}'

near call $CONTRACT draw_near_enter '{
    "lottery_type": "SIMPLE_LOTTERY",
    "num_participants": 5,
    "entry_fee": "'$ONE_NEAR'"
}' --accountId $USER_5 --depositYocto=$ONE_NEAR --gas=$GAS

near view $CONTRACT get_contract_params '{}'
```
```rust
//transfer msg
DrawEnter {
        num_participants: u32,
        lottery_type: String,
        referrer_id: Option<AccountId>
    }
```
#### WITH USN (OR ANOTHER WHITELISTED FT)
near call usdn.testnet ft_transfer_call '{
  "receiver_id": "'$CONTRACT'",
  "amount": "'$ONE_USN'",
  "msg": "{\"DrawEnter\": {\"num_participants\":5, \"lottery_type\":\"SIMPLE_LOTTERY\"}}"
}' --accountId=$USER_5 --depositYocto 1 --gas=200000000000000
near view $CONTRACT get_lottery '{
    "lottery_id": 0
}'


#### OWNER side
```rust
#[payable]
pub fn add_num_participants(
    &mut self, 
    num: u32,
    lottery_type: String
)
#[payable]
pub fn add_entry_fee(&mut self, entry_fee: U128) 
#[payable]
pub fn remove_num_participants(
    &mut self, 
    num: u32,
    lottery_type: String
)
#[payable]
pub fn remove_entry_fee(&mut self, entry_fee: U128) 
#[payable]
pub fn whitelist_token(&mut self, token_id: AccountId)
#[payable]
pub fn remove_whitelist_token(&mut self, token_id: AccountId)
#[payable]
pub fn change_accepted_subs(&mut self, accepted_subs: String) -> bool
```

```sh
near call $CONTRACT change_accepted_subs '{
    "accepted_subs": "sub1.testnet"
}' --accountId $OWNER --depositYocto=1 --gas=$GAS

near call $CONTRACT whitelist_token '{
    "token_id": "usdn.testnet"
}' --accountId $OWNER --depositYocto=1 --gas=$GAS

near call $CONTRACT add_num_participants '{
    "num": 6,
    "lottery_type": "SIMPLE_LOTTERY"
}' --accountId $OWNER --depositYocto=1 --gas=$GAS

near view $CONTRACT get_contract_params '{}'

near call $CONTRACT add_entry_fee '{
    "entry_fee": "'$ONE_USN'"
}' --accountId $OWNER --depositYocto=1 --gas=$GAS

near view $CONTRACT get_contract_params '{}'

near call $CONTRACT remove_entry_fee '{
    "entry_fee": "'$TEN_NEAR'"
}' --accountId $OWNER --depositYocto=1 --gas=$GAS

near view $CONTRACT get_contract_params '{}'

```

#### dev

near call usdn.testnet ft_transfer '{
    "receiver_id": "'$USER_5'",
    "amount": "'$ONE_USN'"
}' --accountId rmlsnk.testnet --depositYocto=1 --gas=$GAS