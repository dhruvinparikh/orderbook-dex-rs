#!/usr/bin/env bash

set -e

if [ "$1" == "--env" -a "$2" == "production" ]; then
kspath=/dnachain/metaverse-keystore/keystore0
acc=acc0
mkdir -p /dnachain/chain/$acc/chains/dna_testnet/keystore
cp $kspath/* /dnachain/chain/$acc/chains/dna_testnet/keystore/
dnachain --bootnodes /ip4/142.93.151.164/tcp/3034/p2p/QmTBup8mUZcNkytxTgz1xWxQNPFQFh5Gnwjdy19D1BPqpd /ip4/142.93.151.164/tcp/3035/p2p/QmSm4yua8ift1AULVisKooPwviVybuRFadr9Rmqurn5DWw  --validator --ws-port 9945 --port 3033 --node-key 92ea498a16084a1e88abf1b3c31b03a545ee608bc2686e64eeb670a237ad427c -d /dnachain/chain/$acc --telemetry-url ws://localhost:8000/submit
else
path=$(pwd)
mkdir -p /tmp/chain/acc0/chains/dna_testnet/keystore
cp -a $path/../metaverse-keystore/keystore0/ /tmp/chain/acc0/chains/dna_testnet/keystore/
$path/target/debug/dnachain --bootnodes /ip4/127.0.0.1/tcp/3034/p2p/QmTBup8mUZcNkytxTgz1xWxQNPFQFh5Gnwjdy19D1BPqpd /ip4/127.0.0.1/tcp/3035/p2p/QmSm4yua8ift1AULVisKooPwviVybuRFadr9Rmqurn5DWw --validator --ws-port 9945 --port 3033 --name "DNA NODE 0" --node-key 92ea498a16084a1e88abf1b3c31b03a545ee608bc2686e64eeb670a237ad427c -d /tmp/chain/acc0 --chain=dna --rpc-cors all --telemetry-url ws://telemetry.mvsdna.com:8000/submit
fi