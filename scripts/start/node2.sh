#!/usr/bin/env bash

set -e

if [ "$1" == "--env" -a "$2" == "production" ]; then
kspath=/dnachain/metaverse-keystore/keystore2
acc=acc2
mkdir -p /dnachain/chain/$acc/chains/dna_testnet/keystore
cp $kspath/* /dnachain/chain/$acc/chains/dna_testnet/keystore/
dnachain --node-key a4143e437a33299676f104135d7bd4b5f0570c0c999d6f9c78a062e65b063161 --validator --ws-port 9946 --port 3035 --bootnodes /ip4/142.93.151.164/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV /ip4/142.93.151.164/tcp/3034/p2p/QmTBup8mUZcNkytxTgz1xWxQNPFQFh5Gnwjdy19D1BPqpd -d /dnachain/chain/$acc --telemetry-url ws://localhost:8000/submit
else
path=$(pwd)
mkdir -p /tmp/chain/acc2/chains/dna_testnet/keystore
cp -a $path/../metaverse-keystore/keystore2/ /tmp/chain/acc2/chains/dna_testnet/keystore/
$path/target/debug/dnachain --node-key a4143e437a33299676f104135d7bd4b5f0570c0c999d6f9c78a062e65b063161 --validator --ws-port 9946 --port 3035 --name "DNA NODE 2" --bootnodes /ip4/127.0.0.1/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV /ip4/127.0.0.1/tcp/3034/p2p/QmTBup8mUZcNkytxTgz1xWxQNPFQFh5Gnwjdy19D1BPqpd -d /tmp/chain/acc2 --chain=dna --telemetry-url ws://telemetry.mvsdna.com:8000/submit
fi