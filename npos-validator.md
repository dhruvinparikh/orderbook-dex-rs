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

# in lib

in `node/cli/src/chain_spec.rs line 170`

```
staking: Some(StakingConfig {
            current_era: 0,
            validator_count: 2,
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        })
```