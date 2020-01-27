#!/usr/bin/env bash

set -e

if [ "$1" == "--env" -a "$2" == "production" ]; then
dnachain --port 30333 -d /dnachain/chain/data-acc0 --ws-port 9944 --rpc-external --ws-external --rpc-cors all --bootnodes /ip4/142.93.151.164/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV --telemetry-url ws://localhost:8000/submit
else
path=$(pwd)
$path/target/debug/dnachain --chain=dna  --port 30333 -d /tmp/chain/data-acc0 --ws-port 9944 --rpc-external --ws-external --rpc-cors all --bootnodes /ip4/127.0.0.1/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV /ip4/127.0.0.1/tcp/3034/p2p/QmTBup8mUZcNkytxTgz1xWxQNPFQFh5Gnwjdy19D1BPqpd /ip4/127.0.0.1/tcp/3035/p2p/QmSm4yua8ift1AULVisKooPwviVybuRFadr9Rmqurn5DWw --telemetry-url --telemetry-url ws://telemetry.mvsdna.com:8000/submit
fi