#!/usr/bin/env bash

path=$(pwd)

cd $path/scripts/transfer && npm --prefer-online install && cd $path
node $path/scripts/transfer/index.js