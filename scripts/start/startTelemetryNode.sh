#!/bin/sh

set -e

if [ "$1" == "--env" -a "$2" == "production" ]; then
/substrate/telemetry/telemetry
fi