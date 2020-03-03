const { WsProvider, Keyring, ApiPromise } = require("@polkadot/api");
const { setSS58Format } = require("@polkadot/util-crypto");
const async = require("async");
const program = require("commander");
const fs = require("fs");
const BN = require("bn.js");
const pkg = require("./package.json");
const { types } = require("./types");

const THRESHOLD_AMOUNT = 20000;
const DECIMALS = 4;

function restoreAccount(accountJson, password) {
  setSS58Format(42);
  const kr = new Keyring();
  const pair = kr.addFromJson(accountJson);
  pair.decodePkcs8(password);
  return pair;
}

async function getApi(url) {
  // Initialise the provider to connect to the local node
  const provider = new WsProvider(url);
  const api = new ApiPromise({ provider, types });
  api.on("disconnected", () => {
    api.disconnect();
  });
  await api.isReady;
  return api;
}

async function getSignTransaction(data) {
  const { to, amount, accountPair, id, api, nonce } = data;
  const promise = new Promise(async (resolve, reject) => {
    try {
      const unsub = await api.tx.balances
        .transfer(to, amount)
        .signAndSend(accountPair, { nonce }, ({ events = [], status }) => {
          if (status.isFinalized) {
            console.log(
              `Transaction included at blockHash ${status.asFinalized}`
            );
            console.log(`Successful id : ${id}`);
            resolve({ status: "SUCCESS", id });
            unsub();
          }
          if (status.isDropped || status.isInvalid || status.isUsurped) {
            console.log(`${id} is FAILURE`);
            resolve({ status: "FAIL", id });
            unsub();
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
    )
    .option(
      "--env <string>",
      "name of environment {<jenkins>}"
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
  let api;
  try {
    api = await getApi(url);
  } catch (e) {
    console.log(`Error connecting to ${url}`);
    process.exit(1);
  }
  const masterAccountPair = getAccountPairFromJSON(masterAccountObj);
  const accountPairArr = accountArr.map(accountObj => {
    const pair = getAccountPairFromJSON(accountObj);
    return pair;
  });
  console.log("Starting bulk-transfer testing suite");
  count = 0;
  await async.mapSeries(accountPairArr, async accountPair => {
    console.log(`Fetching the balance of ${accountPair.address}`);
    let balance;
    balance = await api.query.balances.freeBalance(accountPair.address);
    console.log(
      `Balance of ${accountPair.address} is ${balance
        .div(new BN(10).pow(new BN(DECIMALS)))
        .toString()} DNA/s`
    );
    try {
      if (balance.lt(new BN(THRESHOLD_AMOUNT))) {
        console.log(
          `Funding test account ${accountPair.address} from ${
            masterAccountPair.address
          } amount : ${THRESHOLD_AMOUNT / 10000}`
        );
        const masterAccountNonce = await api.query.system.accountNonce(
          masterAccountPair.address
        );
        const data = {
          to: accountPair.address,
          amount: THRESHOLD_AMOUNT,
          accountPair: masterAccountPair,
          api,
          nonce: masterAccountNonce
        };
        const signTransaction = await getSignTransaction(data);
        console.log(`to : ${data.to} STATUS : ${signTransaction.status}`);
        console.log(
          `Done funding test account => ${accountPair.address} from ${
            masterAccountPair.address
          } amount : ${THRESHOLD_AMOUNT / 10000}`
        );
      }
    } catch (e) {
      console.log(
        `Error while funding account ${accountPair.address} from ${masterAccountPair.address}`
      );
    }
    balance = await api.query.balances.freeBalance(accountPair.address);
    if (balance.gte(new BN(THRESHOLD_AMOUNT))) {
      const randomNum = Math.floor(Math.random() * accountPairArr.length);
      console.log(
        `Should transfer 1 DNA from ${accountPair.address} to ${accountPairArr[randomNum].address}.`
      );
      try {
        const nonce = await api.query.system.accountNonce(accountPair.address);
        const data = {
          to: accountPairArr[randomNum].address,
          amount: 10000,
          accountPair,
          id: ++count,
          api,
          nonce
        };
        const signTransaction = await getSignTransaction(data);
        console.log(
          `from:${accountPair.address}, to:${data.to}, ID : ${signTransaction.id} STATUS : ${signTransaction.status}`
        );
        console.log(
          `Transferred 1 DNA from ${accountPair.address} to ${accountPairArr[randomNum].address}.`
        );
      } catch (e) {
        console.log(
          `Error transferring $${THRESHOLD_AMOUNT / 10000} DNAs from ${
            accountPair.address
          } to ${accountPairArr[randomNum].address}`
        );
      }
    } else {
      console.log(
        `Skipping the transfer from ${
          accountPair.address
        } due to balance less than ${THRESHOLD_AMOUNT / 10000} DNAs`
      );
    }
  });
}

main()
  .catch(console.error)
  .finally(() => process.exit());
