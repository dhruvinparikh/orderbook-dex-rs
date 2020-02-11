# Getting Started

## DNA Transfer

 - `$ npm install`

# Usage

```bash
Usage: dna-transfer [options]

Options:
  -v, --version              output the version
  --url <string>             websocket provider url
  --master-account <string>  account object array {<json-file-path>:<password>}
  --accounts <string>        account object array [{<json-file-path1>:<password1>},{<json-file-path2>:<password2>}
  -h, --help                 output usage information
```

## Example
 ```bash
$ ./scripts/bulk-transfer.sh  --url ws://127.0.0.1:9944 --master-account '{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/master-account.json":"kush1234"}' --accounts '[{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/acc0.json":"kush1234"},{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/acc1.json":"kush1234"},{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/acc2.json":"kush1234"},{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/acc3.json":"kush1234"},{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/acc4.json":"kush1234"}]'
 ```

## DNA Identity

 - `$ npm install`

# Usage

```bash
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
 ```bash
 $ ./scripts/identity.sh  --url ws://127.0.0.1:9944 --sudo-account '{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/sudo-account.json":"kush1234"}' --master-account '{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/master-account.json":"kush1234"}' --registrar '{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/registrar.json":"kush1234"}' --user '{"/Users/dhruvinparikh/blockx-labs/metaverse-dna/test-accounts/user.json":"kush1234"}'
 ```