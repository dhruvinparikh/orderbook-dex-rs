#!/bin/bash

set -e

if [ "$1" == "--env" -a "$2" == "production" ]; then
echo "Stoping all production validator nodes"
# stop all validator nodes
~/.yarn/bin/pm2 stop /dnachain/scripts/pm2process-prod.yml --env production
else
echo "Stoping all development validator nodes"
# start all validator nodes
path=$(pwd)
pm2 stop $path/scripts/pm2process.yml
fi