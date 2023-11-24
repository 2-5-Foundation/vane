#!/bin/bash

# Run cargo build with release flag
cargo build --release

# Check if cargo build was successful
if [ $? -eq 0 ]; then
    # Run vane-node build-spec
    ./target/release/vane-node build-spec --disable-default-bootnode > chain-files/vane-para.json

    # Run vane-node export-genesis-wasm
    ./target/release/vane-node export-genesis-wasm --chain chain-files/vane-para.json > chain-files/vane-para-wasm

    # Run vane-node export-genesis-state
    ./target/release/vane-node export-genesis-state --chain chain-files/vane-para.json > chain-files/vane-para-state

    # Run vane-node build-spec with --raw flag
    ./target/release/vane-node build-spec --dev --disable-default-bootnode --raw > chain-files/vane-tanssi.json

    echo "All commands executed successfully."
else
    echo "Cargo build failed. Please check the build errors."
fi