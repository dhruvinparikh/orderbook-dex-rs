const { WsProvider, Keyring, ApiRx } = require("@polkadot/api");
const async = require("async");
const program = require("commander");
const pkg = require("./package.json");

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
    .option("--keypair-type <string>", "sr25519 or ed25519")
    .option("--amount <number>", "amount");
  program.parse(process.argv);
  if (
    !program.url ||
    !program.keypairType ||
    !program.amount
  ) {
    program.help();
  }
  const masterAccount =
    "grace require cluster fringe insane pulse slab orient palm spot shaft soda";
  const testAccsObj = (testAccounts = {
    "process buyer mixed permit faint bright budget history humor stairs donate table":
      "sr25519",
    "humble mosquito problem small brown aunt caught swallow virus super theory toilet":
      "ed25519",
    "size frown gun bracket present nerve wink glare note sauce replace garbage":
      "sr25519",
    "ceiling fragile wet account retreat future lucky concert supreme stick vault buddy":
      "ed25519",
    "choice kitten boy build canyon below lunar major hire diet daring sword":
      "sr25519"
  });
  return new Promise(resolve => {
    resolve({
      url: program.url,
      masterAccountURI: masterAccount,
      keypairType: program.keypairType,
      testAccounts: testAccsObj,
      amount: program.amount
    });
  });
}

async function main() {
  const {
    url,
    masterAccountURI,
    keypairType,
    testAccounts,
    amount
  } = await getArg();
  const api = await getApi(url);
  const accountPair = await getAccountPair(keypairType, masterAccountURI);
  const { address } = accountPair;
  const testAccURIs = Object.keys(testAccounts).map(x => x);
  console.log("Funding test accounts");
  let count = 0;
  await async.mapSeries(testAccURIs, async uri => {
    const nonce = await api.query.system.accountNonce(address);
    const { address: to } = await getAccountPair(testAccounts[uri], uri);
    const data = { to, amount, accountPair, id: ++count, api, nonce };
    const signTransaction = await getSignTransaction(data);
    console.log(`to : ${to} STATUS : ${signTransaction.status}`);
  });
  console.log("Done funding test accounts");
  console.log("Starting testing suite");
  count = 0;
  let val = 1000;
  await async.mapSeries(testAccURIs, async uri => {
    const accPair = await getAccountPair(testAccounts[uri], uri);
    const { address: from } = accPair;
    const nonce = await api.query.system.accountNonce(from);
    const randomNum = Math.floor(Math.random() * testAccURIs.length);
    const toUri = testAccURIs[randomNum];
    const { address: to } = await getAccountPair(testAccounts[toUri], toUri);
    const data = {
      to,
      amount: val,
      accountPair: accPair,
      id: ++count,
      api,
      nonce
    };
    const signTransaction = await getSignTransaction(data);
    console.log(
      `from:${from}, to:${to}, ID : ${signTransaction.id} STATUS : ${signTransaction.status}`
    );
  });
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
