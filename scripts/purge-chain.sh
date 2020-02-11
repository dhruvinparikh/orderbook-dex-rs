#!/usr/bin/env bash
path=$(pwd)

if [ "$1" == "--env" -a "$2" == "production" ]; then
dnachain purge-chain --chain=dna -d /dnachain/chain/acc0 -y 
dnachain purge-chain --chain=dna -d /dnachain/chain/acc1 -y
dnachain purge-chain --chain=dna -d /dnachain/chain/acc2 -y
dnachain purge-chain --chain=dna -d /dnachain/chain/data-acc0 -y
else
$path/target/debug/dnachain purge-chain --chain=local -d /tmp/chain/acc0 -y
$path/target/debug/dnachain purge-chain --chain=local -d /tmp/chain/acc1 -y
$path/target/debug/dnachain purge-chain --chain=local -d /tmp/chain/acc2 -y
$path/target/debug/dnachain purge-chain --chain=local -d /tmp/chain/data-acc0 -y
fi