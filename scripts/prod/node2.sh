#!/usr/bin/env bash
set -e
kspath=/dnachain/metaverse-keystore/keystore2
acc=acc2

mkdir -p /dnachain/chain/$acc/chains/dna_testnet/keystore
cp $kspath/* /dnachain/chain/$acc/chains/dna_testnet/keystore/

dnachain --validator --ws-port 9946 --port 3035 --bootnodes /ip4/142.93.151.164/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV -d /dnachain/chain/$acc --chain /dnachain/build-spec.dna.json --telemetry-url ws://localhost:8000/submit