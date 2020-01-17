#!/usr/bin/env bash

set -e

if [ "$1" == "--env" -a "$2" == "production" ]; then
kspath=/dnachain/metaverse-keystore/keystore1
acc=acc1
mkdir -p /dnachain/chain/$acc/chains/dna_testnet/keystore
cp $kspath/* /dnachain/chain/$acc/chains/dna_testnet/keystore/
dnachain --validator --ws-port 9946 --port 3034 --node-key 569256cbfd3692f68085c34b19b82dd2c5951df4a802cd714b3ea798b83fe648 --bootnodes /ip4/142.93.151.164/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV /ip4/142.93.151.164/tcp/3035/p2p/QmSm4yua8ift1AULVisKooPwviVybuRFadr9Rmqurn5DWw -d /dnachain/chain/$acc --telemetry-url ws://localhost:8000/submit
else
path=$(pwd)
mkdir -p /tmp/chain/acc1/chains/dna_testnet/keystore
cp -a $path/../metaverse-keystore/keystore1/ /tmp/chain/acc1/chains/dna_testnet/keystore/
$path/target/debug/dnachain --validator --ws-port 9946 --port 3034 --node-key 569256cbfd3692f68085c34b19b82dd2c5951df4a802cd714b3ea798b83fe648 --name "DNA NODE 1" -d /tmp/chain/acc1 --bootnodes /ip4/127.0.0.1/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV /ip4/127.0.0.1/tcp/3035/p2p/QmSm4yua8ift1AULVisKooPwviVybuRFadr9Rmqurn5DWw --chain=dna --telemetry-url --telemetry-url ws://telemetry.mvsdna.com:8000/submit
fi