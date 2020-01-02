#!/usr/bin/env bash

path=$(pwd)

mkdir /tmp/acc0
mkdir /tmp/acc0/chains
mkdir /tmp/acc0/chains/dna_testnet
mkdir /tmp/acc0/chains/dna_testnet/keystore
cp -a $path/keystore/ /tmp/acc0/chains/dna_testnet/keystore/

$path/target/debug/dnachain --validator --node-key 92ea498a16084a1e88abf1b3c31b03a545ee608bc2686e64eeb670a237ad427c -d /tmp/acc0 --chain $path/dna1.json
