# DNA Transfer

# Getting Started

 - `$ npm install`

# Usage

```
Usage: dna-transfer [options]

Options:
  -v, --version            output the version
  --url <string>           websocket provider url
  --keypair-type <string>  sr25519 or ed25519
  --amount <number>        amount
  -h, --help               output usage information
```

## Example
 `./scripts/bulk-transfer.sh --url "ws://127.0.0.1:9944" --keypair-type "sr25519" --amount 1000000`