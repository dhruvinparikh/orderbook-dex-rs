#!/usr/bin/env bash

path=$(pwd)

mkdir /tmp/acc2
mkdir /tmp/acc2/chains
mkdir /tmp/acc2/chains/dna_testnet
mkdir /tmp/acc2/chains/dna_testnet/keystore
cp -a $path/keystore2/ /tmp/acc2/chains/dna_testnet/keystore/

$path/target/debug/dnachain --validator --name "DNA NODE 2" --bootnodes /ip4/192.168.0.13/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV -d /tmp/acc2 --chain $path/dnachainspec.json