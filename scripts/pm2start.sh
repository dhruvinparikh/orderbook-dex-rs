#!/bin/bash

set -e

if [ "$1" == "--env" -a "$2" == "production" ]; then
echo "Starting all production validator nodes"
# start all validator nodes
~/.yarn/bin/pm2 start /dnachain/scripts/pm2process-prod.yml --env production
else
echo "Starting all development validator nodes"
# start all validator nodes
path=$(pwd)
pm2 start $path/scripts/pm2process.yml
fi
