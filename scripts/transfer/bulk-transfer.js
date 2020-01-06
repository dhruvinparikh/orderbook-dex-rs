const { WsProvider, Keyring, ApiRx } = require("@polkadot/api");
const async = require("async");
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

async function main() {
  const address = "5ChnoXHLockcTcB1v6JMTa7H6cJZ7zFwPqBDHT6K8df4fc2N";
  const url = process.argv[3] || "ws://127.0.0.1:9944";
  const seedWords =
    "grace require cluster fringe insane pulse slab orient palm spot shaft soda";
  const keypairType = "sr25519";
  //   const to = "5GiFdKY55j8EwviqBVi2Jd6MNb6GnZDRZnc2tVSENmEdn1Vw";
  const api = await getApi(url);
  const accountPair = await getAccountPair(keypairType, seedWords);
  const signTxnPromises = [];
  const amount = 2000;
  const tos = [
    "5GiFdKY55j8EwviqBVi2Jd6MNb6GnZDRZnc2tVSENmEdn1Vw",
    "5CW139xu5PGYjccoJ8aKJRkD8BBRM5cHvLnfVm9Au2NbkTDw",
    "5CW139xu5PGYjccoJ8aKJRkD8BBRM5cHvLnfVm9Au2NbkTDw",
    "5H6W77mdpU31V6vEZeMWcCaz14cKDrjqwFiXDY9ZzNX1w81x",
    "5H6W77mdpU31V6vEZeMWcCaz14cKDrjqwFiXDY9ZzNX1w81x"
  ];
  let count = 0;
  await async.mapSeries(tos, async to => {
    const nonce = await api.query.system.accountNonce(address);
    const data = { to, amount, accountPair, id: ++count, api, nonce };
    const signTransaction = await getSignTransaction(data);
    console.log(
      `ID : ${signTransaction.id} STATUS : ${signTransaction.status}`
    );
  });
  process.exit(0);
}

main().catch(console.error);
