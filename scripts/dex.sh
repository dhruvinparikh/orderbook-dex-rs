#!/usr/bin/env bash

path=$(pwd)

cd $path/scripts/dex && npm --prefer-online install && cd $path
node $path/scripts/dex/index.js $@