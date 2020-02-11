# Running atleast one validator with rotating session keys

## Getting Started
 * Start validator with `./target/release/dnachain --bob --base-path /tmp/bob --port 3033 --validator --ws-port 9944`
 * Start second validator with `./target/release/dnachain --alice --base-path /tmp/alice --port 3034 --validator --bootnodes /ip4/##bob_ip#/tcp/30333/p2p/##bob_node_identifier## --ws-port 9945`
 * Both should start producing blocks
 * Run a regular validator with --validator option. Also run ws-port which has dep to second step
 * Connect PDOTJS APP to your local node
 * Create 2 accounts. 1 Controller & 1 Stash
 * Fill Stash and Controlller with funds
 * Extrinsics → Extrnsic ( staking) → Function ( bond ) → Choose correct ‘controller’ → value :  selection something → payee ( staked )
 * Get correct session key. PDOTJS APP → Extrinsics → Extrinsic (author) → function (rotateKeys() → Submit. Returns Session Key
 * Add Session key. Staking → Account Card → Account Gear Wheel → Add Session Key → Complete this call
 * Start validation step. Staking → Account Card → Click Validate
 * Wait for Session to complete. Should valdiator on next session


# Staking Configuration

in `node/cli/src/chain_spec.rs line 170`

```
staking: Some(StakingConfig {
            current_era: 0,
            validator_count: 1,
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

# Running Chain First Time

## Number of Validators
Configure No of Validators to be 2 for generation of blocks and 3 for finalization.

## Set Keys
In `chain_spec.rs` you have Initial Authorities defined. Add them to your Node's keystore.

## Start Validators
Run validators as depicted above and it should start generating and finalizing blocks.


