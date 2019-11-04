# Rust Version

* cargo 1.38.0 (23ef9a4ef 2019-08-20)

```curl https://sh.rustup.rs -sSf | sh```

# Substrate version (not needed if using locally)

* substrate 2.0.0-4a13f614c-x86_64-macos

# Start new blockchain project 

* ```substrate-node-new <node-name> <author>```
* Or take code from github --> cargo build --release 

# Run blockchain, in folder 

* ```./target/release/<node-name> --dev```

# Documentation links 

* https://substrate.dev/docs/en/tutorials/start-a-private-network-with-substrate

# For some reason

* Making changes to chain_spec.rs requires you to purge the chain, delete your target folder and rebuild

# Configuration

* We should locally genetrate our config file with Rust and pass it off to people to build, thus keeping our secrets 
* we should have a skeleton confi_spec.rs
* ```./target/release/<node-name> build-spec --chain=local > customSpec.json```
* Our config file best practice should be raw
* ```./target/release/<node-name> build-spec --chain customSpec.json --raw > customSpecRaw.json```

* Wasm is not deterministic with compilation so chain spec should be shared
* to launch like this, same steps but --chain ./customSpecRaw.json 


for me 

node 1 
./target/release/dna --chain=prod --base-path /tmp/prod --key "budget number index moon visa midnight process answer large panther tenant appear" --validator --port 30333 --name temp

node 2 

./target/release/dna   --base-path /tmp/prod2   --chain=prod  --key 'coconut session disorder bone spot tattoo uncover basket basic laundry glad shiver'   --port 30334   --validator   --name node2   --bootnodes /ip4/127.0.0.1/tcp/303339/p2p/QmerFAhkTD8Rgj1aJFQWWDnQq2bR7SFxzuoHnPrpeKBzEd ----ws-port 9945


node 3 

./target/release/dna   --base-path /tmp/prod3   --chain=prod  --key 'debate convince invite virus shy tank swift fuel aerobic open alien address'   --port 30336   --valator   --name node3   --bootnodes /ip4/127.0.0.1/tcp/303339/p2p/QmerFAhkTD8Rgj1aJFQWWDnQq2bR7SFxzuoHnPrpeKBzEd /ip4/192.168.1.166/tcp/30334/p2p/QmRbfyF3AtWB2HiMYxziY6f1FfKM2GkSDEtj8FFFsdQ4BV --ws-port 9946 



node chainspec
 ./target/release/dna   --base-path /tmp/prod2   --chain ./customSpec.json   --key //Bob   --port 30333   --validator   --name BobsNode   --bootnodes /ip4/127.0.0.1/tcp/303339/p2p/QmP1d9BCxcKW46R4uFoViPJZuJNdktqLya2cVp3csbaRFt


Babe and granpa 
https://github.com/airalab/substrate-node-robonomics/blob/master/node/cli/src/chain_spec.rs#L157