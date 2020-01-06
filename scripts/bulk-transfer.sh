#!/usr/bin/env bash

path=$(pwd)

cd $path/scripts/transfer && npm install && cd $path
node $path/scripts/transfer/bulk-transfer.js