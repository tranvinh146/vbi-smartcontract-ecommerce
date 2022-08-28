#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
mkdir out
cp target/wasm32-unknown-unknown/release/*.wasm out/contract.wasm