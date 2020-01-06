#!/usr/bin/env bash

path=$(pwd)

cd $path/scripts/transfer && npm install && cd $path
if [ $# -eq 0 ]
then
node $path/scripts/transfer/bulk-transfer.js 
else
node $path/scripts/transfer/bulk-transfer.js $1 $2
fi

