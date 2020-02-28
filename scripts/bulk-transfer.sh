#!/usr/bin/env bash

path=$(pwd)

cd $path/scripts/transfer
rm -rf node_modules
rm -rf package-lock.json
npm clean
if [ "$1" == "--env" -a "$2" == "jenkins" ]; then
echo "Starting installing deps in jenkins box."
cp -aRf ./kush/metaverse-dna-transfer/ .
else
echo "Starting installing deps."
fi
npm install
if [ "$1" == "--env" -a "$2" == "jenkins" ]; then
echo "Finished installing dependencies in jenkins box."
cp -aRf node_modules ./kush/metaverse-dna-transfer/
cp -aRf package-lock.json ./kush/metaverse-dna-transfer/
else
echo "Finished installing deps."
fi 
cd $pat
node $path/scripts/transfer/bulk-transfer.js $@

