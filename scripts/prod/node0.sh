#!/usr/bin/env bash
set -e
kspath=/dnachain/metaverse-keystore/keystore0
acc=acc0

mkdir -p /dnachain/chain/$acc/chains/dna_testnet/keystore
cp $kspath/* /dnachain/chain/$acc/chains/dna_testnet/keystore/

dnachain --validator --ws-port 9945 --port 3033 --node-key 92ea498a16084a1e88abf1b3c31b03a545ee608bc2686e64eeb670a237ad427c -d /dnachain/chain/$acc --chain /dnachain/build-spec.dna.json --telemetry-url ws://localhost:8000/submit