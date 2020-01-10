#!/usr/bin/env bash

path=$(pwd)

mkdir -p /tmp/acc1/chains/dna_testnet/keystore
cp -a $path/../metaverse-keystore/keystore1/ /tmp/acc1/chains/dna_testnet/keystore/

$path/target/debug/dnachain --validator --name "DNA NODE 1" -d /tmp/acc1 --bootnodes /ip4/192.168.0.13/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV --chain $path/build-spec.dna.json 