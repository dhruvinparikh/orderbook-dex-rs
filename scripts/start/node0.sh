#!/usr/bin/env bash

path=$(pwd)

mkdir /tmp/acc0
mkdir /tmp/acc0/chains
mkdir /tmp/acc0/chains/dna_testnet
mkdir /tmp/acc0/chains/dna_testnet/keystore
cp -a $path/keystore0/ /tmp/acc0/chains/dna_testnet/keystore/

$path/target/debug/dnachain --validator --name "DNA NODE 0" --node-key 92ea498a16084a1e88abf1b3c31b03a545ee608bc2686e64eeb670a237ad427c -d /tmp/acc0 --chain $path/dnachainspec.json
