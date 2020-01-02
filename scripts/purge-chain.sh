path=$(pwd)

$path/target/debug/dnachain purge-chain -d /tmp/node0 -y
$path/target/debug/dnachain purge-chain -d /tmp/node1 -y
$path/target/debug/dnachain purge-chain -d /tmp/node2 -y