#!/usr/bin/env bash
path=$(pwd)

$path/target/debug/dnachain purge-chain -d /tmp/acc0 -y
$path/target/debug/dnachain purge-chain -d /tmp/acc1 -y
$path/target/debug/dnachain purge-chain -d /tmp/acc2 -y