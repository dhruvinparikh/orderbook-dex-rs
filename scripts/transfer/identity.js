const { WsProvider, Keyring, ApiRx } = require("@polkadot/api");
const { setSS58Format } = require("@polkadot/util-crypto");
const program = require("commander");
const pkg = require("./package.json");
const fs = require("fs");

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
  const api = await ApiRx.create({ provider }).toPromise();
  return api;
}

async function transfer(data) {
  const { to, amount, accountPair, api, nonce } = data;
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
            console.log(`Successful`);
            resolve({ status: "SUCCESS"});
            subscription.unsubscribe();
          }
          if (status.isDropped || status.isInvalid || status.isUsurped) {
            console.log(`FAILURE`);
            resolve({ status: "FAIL"});
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
      "master account object {<json-file-path>:<password>}"
    )
    .option(
      "--registrar <string>",
      "registrar object {<json-file-path>:<password>}"
    )
    .option("--user <string>", "user object {<json-file-path>:<password>}");
  program.parse(process.argv);
  if (
    !program.url ||
    !program.masterAccount ||
    !program.registrar ||
    !program.user
  ) {
    program.help();
  }
  return new Promise(resolve => {
    resolve({
      url: program.url,
      masterAccountObj: JSON.parse(program.masterAccount),
      registrarObj: JSON.parse(program.registrar),
      userObj: JSON.parse(program.user)
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
  const { url, masterAccountObj, registrarObj, userObj } = await getArg();
  const api = await getApi(url);
  const masterAccountPair = getAccountPairFromJSON(masterAccountObj);
  const registrarAccountPair = getAccountPairFromJSON(registrarObj);
  const userAccountPair = getAccountPairFromJSON(userObj);

  let masterAccountNonce = await api.query.system.accountNonce(
    masterAccountPair.address
  );
  console.log("Funding registrar account.");
  const registrarTx = await transfer({
    to: registrarAccountPair.address,
    amount: 1000000,
    accountPair: masterAccountPair,
    api,
    nonce: masterAccountNonce
  });
  console.log("Done funding registrar account.", registrarTx);
  masterAccountNonce = await api.query.system.accountNonce(
    masterAccountPair.address
  );
  console.log("Funding user account.");
  const userTx = await transfer({
    to: userAccountPair.address,
    amount: 1000000,
    accountPair: masterAccountPair,
    api,
    nonce: masterAccountNonce
  });
  console.log("Done funding user account.", userTx);
  //   const accountPair = await getAccountPair(keypairType, masterAccountURI);
  //   const { address } = accountPair;
  //   const testAccURIs = Object.keys(testAccounts).map(x => x);
  //   console.log("Funding test accounts");
  //   let count = 0;
  //   await async.mapSeries(testAccURIs, async uri => {
  //     const nonce = await api.query.system.accountNonce(address);
  //     const { address: to } = await getAccountPair(testAccounts[uri], uri);
  //     const data = { to, amount, accountPair, id: ++count, api, nonce };
  //     const signTransaction = await getSignTransaction(data);
  //     console.log(`to : ${to} STATUS : ${signTransaction.status}`);
  //   });
  //   console.log("Done funding test accounts");
  //   console.log("Starting testing suite");
  //   count = 0;
  //   let val = 1000;
  //   await async.mapSeries(testAccURIs, async uri => {
  //     const accPair = await getAccountPair(testAccounts[uri], uri);
  //     const { address: from } = accPair;
  //     const nonce = await api.query.system.accountNonce(from);
  //     const randomNum = Math.floor(Math.random() * testAccURIs.length);
  //     const toUri = testAccURIs[randomNum];
  //     const { address: to } = await getAccountPair(testAccounts[toUri], toUri);
  //     const data = {
  //       to,
  //       amount: val,
  //       accountPair: accPair,
  //       id: ++count,
  //       api,
  //       nonce
  //     };
  //     const signTransaction = await getSignTransaction(data);
  //     console.log(
  //       `from:${from}, to:${to}, ID : ${signTransaction.id} STATUS : ${signTransaction.status}`
  //     );
  //   });
  // const tos = [
  //   "5GiFdKY55j8EwviqBVi2Jd6MNb6GnZDRZnc2tVSENmEdn1Vw",
  //   "5CW139xu5PGYjccoJ8aKJRkD8BBRM5cHvLnfVm9Au2NbkTDw",
  //   "5CW139xu5PGYjccoJ8aKJRkD8BBRM5cHvLnfVm9Au2NbkTDw",
  //   "5H6W77mdpU31V6vEZeMWcCaz14cKDrjqwFiXDY9ZzNX1w81x",
  //   "5H6W77mdpU31V6vEZeMWcCaz14cKDrjqwFiXDY9ZzNX1w81x"
  // ];
  process.exit(0);
}

main().catch(console.error);
