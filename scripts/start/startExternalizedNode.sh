#!/bin/sh

set -e

if [ "$1" == "--env" -a "$2" == "production" ]; then
dnachain --chain /dnachain/build-spec.dna.json  --port 30333 -d /dnachain/chain/data-acc0 --ws-port 9944 --rpc-external --ws-external --rpc-cors all --bootnodes /ip4/142.93.151.164/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV --telemetry-url ws://localhost:8000/submit
else
path=$(pwd)
$path/target/debug/dnachain --chain $path/build-spec.dna.json  --port 30333 -d /tmp/chain/data-acc0 --ws-port 9944 --rpc-external --ws-external --rpc-cors all --bootnodes /ip4/192.168.0.13/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV --telemetry-url --telemetry-url ws://telemetry.mvsdna.com:8000/submit
fi