#!/usr/bin/env bash

path=$(pwd)

cd $path/scripts/transfer
rm -rf node_modules
rm -rf package-lock.json
npm install 
cd $pat
node $path/scripts/transfer/bulk-transfer.js $@

