const { WsProvider, Keyring, ApiRx } = require("@polkadot/api");
const { setSS58Format } = require("@polkadot/util-crypto");
const async = require("async");
const program = require("commander");
const fs = require("fs");
const pkg = require("./package.json");

function restoreAccount(accountJson, password) {
  setSS58Format(42);
  const kr = new Keyring();
  const pair = kr.addFromJson(accountJson);
  pair.decodePkcs8(password);
  return pair;
}

async function getAccountPair(keypairType, seedWords) {
  const keyring = new Keyring({ type: keypairType });
  const accountPair = keyring.addFromUri(seedWords);
  return accountPair;
}

async function getApi(url) {
  // Initialise the provider to connect to the local node
  const provider = new WsProvider(url);

  // Create the API and wait until ready
  const api = await ApiRx.create({
    provider,
    types: { DNAi64: "Option<i64>" }
  }).toPromise();
  return api;
}

async function getSignTransaction(data) {
  const { to, amount, accountPair, id, api, nonce } = data;
  const promise = new Promise(async (resolve, reject) => {
    try {
      const subscription = await api.tx.balances
        .transfer(to, amount)
        .signAndSend(accountPair, nonce)
        .subscribe(({ events = [], status }) => {
          if (status.isFinalized) {
            console.log(
              `Transaction included at blockHash ${status.asFinalized}`
            );
            console.log(`Successful id : ${id}`);
            resolve({ status: "SUCCESS", id });
            subscription.unsubscribe();
          }
          if (status.isDropped || status.isInvalid || status.isUsurped) {
            console.log(`${id} is FAILURE`);
            resolve({ status: "FAIL", id });
          }
        });
    } catch (e) {
      console.log("Error : ", e);
      reject(e);
    }
  });

  return promise;
}

async function getArg() {
  program
    .version(pkg.version, "-v, --version", "output the version")
    .name(pkg.name)
    .option("--url <string>", "websocket provider url")
    .option(
      "--master-account <string>",
      "account object array {<json-file-path>:<password>}"
    )
    .option(
      "--accounts <string>",
      "account object array [{<json-file-path1>:<password1>},{<json-file-path2>:<password2>}"
    );
  program.parse(process.argv);
  if (!program.url || !program.masterAccount || !program.accounts) {
    program.help();
  }
  return new Promise(resolve => {
    resolve({
      url: program.url,
      masterAccountObj: JSON.parse(program.masterAccount),
      accountArr: JSON.parse(program.accounts)
    });
  });
}

function getAccountPairFromJSON(accountObj) {
  const account = JSON.parse(fs.readFileSync(Object.keys(accountObj)[0]));
  const accountPair = restoreAccount(
    account,
    accountObj[Object.keys(accountObj)[0]]
  );
  return accountPair;
}

async function main() {
  const { url, masterAccountObj, accountArr } = await getArg();
  const api = await getApi(url);
  const masterAccountPair = getAccountPairFromJSON(masterAccountObj);
  const accountPairArr = accountArr.map(accountObj => {
    const pair = getAccountPairFromJSON(accountObj);
    return pair;
  });
  console.log("Funding test accounts");
  let count = 0;
  await async.mapSeries(accountPairArr, async accountPair => {
    const nonce = await api.query.system.accountNonce(
      masterAccountPair.address
    );
    const data = {
      to: accountPair.address,
      amount: 10000000,
      accountPair: masterAccountPair,
      id: ++count,
      api,
      nonce
    };
    const signTransaction = await getSignTransaction(data);
    console.log(`to : ${data.to} STATUS : ${signTransaction.status}`);
  });
  console.log("Done funding test accounts");
  console.log("Starting testing suite");
  count = 0;
  let val = 1000;
  await async.mapSeries(accountPairArr, async accountPair => {
    const nonce = await api.query.system.accountNonce(accountPair.address);
    const randomNum = Math.floor(Math.random() * accountPairArr.length);
    const data = {
      to: accountPairArr[randomNum].address,
      amount: val,
      accountPair,
      id: ++count,
      api,
      nonce
    };
    const signTransaction = await getSignTransaction(data);
    console.log(
      `from:${accountPair.address}, to:${data.to}, ID : ${signTransaction.id} STATUS : ${signTransaction.status}`
    );
  });
  process.exit(0);
}

main().catch(console.error);
