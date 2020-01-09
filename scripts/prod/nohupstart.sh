#!/bin/bash

set -e

echo "Starting all validator nodes"
# start all validator nodes
nohup /dnachain/scripts/start/node0.sh > /dnachain/logs/validator-node-0.log 2>&1 &
nohup /dnachain/scripts/start/node1.sh > /dnachain/logs/validator-node-1.log 2>&1 &
nohup /dnachain/scripts/start/node2.sh > /dnachain/logs/validator-node-2.log 2>&1 &

echo "Starting externalized node"
# start all externalized nodes
nohup /dnachain/startExternalizedNode.sh > /dnachain/logs/externalized-node.log 2>&1 &
nohup /dnachain/startTelemetryNode.sh > /dnachain/logs/telemetry-node.log 2>&1 &
