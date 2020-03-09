# Getting Started

## DNA Dex

 - `$ npm install`

# Usage

```bash
Usage: dex [options]

Options:
  -v, --version              output the version
  --url <string>             websocket provider url
  --master-account <string>  account object array {<json-file-path>:<password>}
  --issuer <string>          account object {<json-file-path1>:<password1>}
  --trader <string>          account object {<json-file-path1>:<password1>}
  -h, --help                 output usage information
```

## Example
 ```bash
$ ./scripts/dex.sh  --url ws://127.0.0.1:9944 --master-account '{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/master-account.json":"kush1234"}' --issuer '{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/acc0.json":"kush1234"}' --trader '{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/acc1.json":"kush1234"}'
 ```