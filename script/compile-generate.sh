#!/bin/bash

# Run cargo build with release flag
cargo build --release

# Check if cargo build was successful
if [ $? -eq 0 ]; then
    # Run vane-node build-spec
    ./target/release/vane-tanssi-node build-spec --parachain-id 2000 > chain_files/vane-tanssi.json

    # Run vane-node build-spec --raw
    ./target/release/vane-tanssi-node build-spec --chain chain_files/vane-tanssi.json --raw > chain_files/vane-tanssi-raw.json

    echo "All commands executed successfully."
else
    echo "Cargo build failed. Please check the build errors."
fi