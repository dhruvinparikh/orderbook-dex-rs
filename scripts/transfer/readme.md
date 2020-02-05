# Getting Started

## DNA Transfer

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

## DNA Identity

 - `$ npm install`

# Usage

```
Usage: dna-transfer [options]

Options:
-v, --version              output the version
  --sudo-account <string>    sudo account object {<json-file-path>:<password>}
  --url <string>             websocket provider url
  --master-account <string>  master account object {<json-file-path>:<password>}
  --registrar <string>       registrar object {<json-file-path>:<password>}
  --user <string>            user object {<json-file-path>:<password>}
  -h, --help                 output usage information
```

## Example
 ```
 ./scripts/identity.sh  --url ws://127.0.0.1:9944 --sudo-account '{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/sudo-account.json":"kush1234"}' --master-account '{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/master-account.json":"kush1234"}' --registrar '{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/registrar.json":"kush1234"}' --user '{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/user.json":"kush1234"}'
 ```