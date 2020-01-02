#!/usr/bin/env bash

path=$(pwd)

mkdir /tmp/acc1
mkdir /tmp/acc1/chains
mkdir /tmp/acc1/chains/dna_testnet
mkdir /tmp/acc1/chains/dna_testnet/keystore
cp -a $path/keystore/ /tmp/acc1/chains/dna_testnet/keystore/

$path/target/debug/dnachain --validator -d /tmp/acc1 --bootnodes /ip4/192.168.0.13/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV --chain $path/dna1.json 
