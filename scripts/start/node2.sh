#!/usr/bin/env bash

set -e

if [ "$1" == "--env" -a "$2" == "production" ]; then
kspath=/dnachain/metaverse-keystore/keystore2
acc=acc2
mkdir -p /dnachain/chain/$acc/chains/dna_testnet/keystore
cp $kspath/* /dnachain/chain/$acc/chains/dna_testnet/keystore/
dnachain --node-key a4143e437a33299676f104135d7bd4b5f0570c0c999d6f9c78a062e65b063161 --validator --ws-port 9946 --port 3035 --bootnodes /ip4/142.93.151.164/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV -d /dnachain/chain/$acc --chain /dnachain/build-spec.dna.json --telemetry-url ws://localhost:8000/submit
else
path=$(pwd)
mkdir -p /tmp/acc2/chains/dna_testnet/keystore
cp -a $path/../metaverse-keystore/keystore2/ /tmp/acc2/chains/dna_testnet/keystore/
$path/target/debug/dnachain --node-key a4143e437a33299676f104135d7bd4b5f0570c0c999d6f9c78a062e65b063161 --validator --ws-port 9946 --port 3035 --name "DNA NODE 2" --bootnodes /ip4/192.168.0.13/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV -d /tmp/acc2 --chain $path/build-spec.dna.json --telemetry-url ws://telemetry.mvsdna.com:8000/submit
fi