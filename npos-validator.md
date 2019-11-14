# Running atleast validator with rotating session keys

## Getting Started

- Open new terminal window.
- Run `git clone https://dhruvinparikh@bitbucket.org/apigarage-core/metaverse-dna.git` to clone this repo.
- Run `./scripts/getgoing.sh` to install Rust.
- 
- Run `./scripts/init.sh` to initialise WASM build environment.
- Run `./scripts/update.sh` to build WASM binaries.
- Run `cargo build --release` to build the target using WASM binaries.
- Run `./target/release/substrate --chain ./customRaw.json --bob --base-path /tmp/bob --port 30333` to start a node with inbuilt validator.
```
Note : Copy the node key from the logs that appear on console.
Use command ifconfig to get the machine network ip.
```
- Open another terminal window and run `./target/release/substrate --chain ./customRaw.json --base-path /tmp/bob1 --bootnodes /ip4/<validator-node-ip>/tcp/30333/p2p/<node-key>`.
