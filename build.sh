#!/bin/bash
set -e

#testnet build
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/*.wasm ./res/

#mainnet build
# export NEAR_ENV=mainnet
# RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release --no-default-features
# cp target/wasm32-unknown-unknown/release/*.wasm ./res/