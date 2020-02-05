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
  const api = await ApiRx.create({
    provider,
    types: { DNAi64: "Option<i64>" }
  }).toPromise();
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
            resolve({ status: "SUCCESS" });
            subscription.unsubscribe();
          }
          if (status.isDropped || status.isInvalid || status.isUsurped) {
            console.log(`FAILURE`);
            resolve({ status: "FAIL" });
          }
        });
    } catch (e) {
      console.log("Error : ", e);
      reject(e);
    }
  });
  return promise;
}
async function addRegistar(data) {
  const { accountPair, registrarAccountPair, api, nonce } = data;
  const promise = new Promise(async (resolve, reject) => {
    try {
      const subscription = await api.tx.sudo
        .sudo(api.tx.identity.addRegistrar(registrarAccountPair.accountId))
        .signAndSend(accountPair, nonce)
        .subscribe(({ events = [], status }) => {
          if (status.isFinalized) {
            console.log(
              `Transaction included at blockHash ${status.asFinalized}`
            );
            console.log(`Successful`);
            resolve({ status: "SUCCESS" });
            subscription.unsubscribe();
          }
          if (status.isDropped || status.isInvalid || status.isUsurped) {
            console.log(`FAILURE`);
            resolve({ status: "FAIL" });
          }
        });
    } catch (e) {
      console.log("Error : ", e);
      reject(e);
    }
  });

  return promise;
}

async function setIdentity(data) {
  const { accountPair, api, nonce, info } = data;
  const promise = new Promise(async (resolve, reject) => {
    try {
      const subscription = await api.tx.identity
        .setIdentity(info)
        .signAndSend(accountPair, nonce)
        .subscribe(({ events = [], status }) => {
          if (status.isFinalized) {
            console.log(
              `Transaction included at blockHash ${status.asFinalized}`
            );
            console.log(`Successful`);
            resolve({ status: "SUCCESS" });
            subscription.unsubscribe();
          }
          if (status.isDropped || status.isInvalid || status.isUsurped) {
            console.log(`FAILURE`);
            resolve({ status: "FAIL" });
          }
        });
    } catch (e) {
      console.log("Error : ", e);
      reject(e);
    }
  });

  return promise;
}

async function requestJudgement(data) {
  const { accountPair, api, nonce,reg_index,max_fee } = data;
  const promise = new Promise(async (resolve, reject) => {
    try {
      const subscription = await api.tx.identity
        .requestJudgement(reg_index,max_fee)
        .signAndSend(accountPair, nonce)
        .subscribe(({ events = [], status }) => {
          if (status.isFinalized) {
            console.log(
              `Transaction included at blockHash ${status.asFinalized}`
            );
            console.log(`Successful`);
            resolve({ status: "SUCCESS" });
            subscription.unsubscribe();
          }
          if (status.isDropped || status.isInvalid || status.isUsurped) {
            console.log(`FAILURE`);
            resolve({ status: "FAIL" });
          }
        });
    } catch (e) {
      console.log("Error : ", e);
      reject(e);
    }
  });

  return promise;
}

async function provideJudgement(data) {
  const { accountPair, api, nonce,reg_index,userAccountPair,judgement } = data;
  const promise = new Promise(async (resolve, reject) => {
    try {
      const subscription = await api.tx.identity
        .provideJudgement(reg_index,userAccountPair.address.toString(),judgement)
        .signAndSend(accountPair, nonce)
        .subscribe(({ events = [], status }) => {
          if (status.isFinalized) {
            console.log(
              `Transaction included at blockHash ${status.asFinalized}`
            );
            console.log(`Successful`);
            resolve({ status: "SUCCESS" });
            subscription.unsubscribe();
          }
          if (status.isDropped || status.isInvalid || status.isUsurped) {
            console.log(`FAILURE`);
            resolve({ status: "FAIL" });
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
    .option(
      "--sudo-account <string>",
      "sudo account object {<json-file-path>:<password>}"
    )
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
    !program.sudoAccount ||
    !program.masterAccount ||
    !program.registrar ||
    !program.user
  ) {
    program.help();
  }
  return new Promise(resolve => {
    resolve({
      url: program.url,
      sudoAccountObj: JSON.parse(program.sudoAccount),
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
  const {
    url,
    sudoAccountObj,
    masterAccountObj,
    registrarObj,
    userObj
  } = await getArg();
  const api = await getApi(url);
  const masterAccountPair = getAccountPairFromJSON(masterAccountObj);
  const registrarAccountPair = getAccountPairFromJSON(registrarObj);
  const userAccountPair = getAccountPairFromJSON(userObj);
  const sudoAccountPair = getAccountPairFromJSON(sudoAccountObj);
  let userAccountNonce,masterAccountNonce,sudoAccountNonce,registrarAccountNonce;
  masterAccountNonce = await api.query.system.accountNonce(
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
  console.log("Set on-chain identity for registrar");
  const registrarInfo = {
    additional: [],
    display: { Raw: "0x4b555348" },
    legal: { None: null },
    web: { None: null },
    riot: { None: null },
    email: { None: null },
    pgpFingerprint: null,
    image: { None: null },
    twitter: { None: null }
  };
  registrarAccountNonce = await api.query.system.accountNonce(
    registrarAccountPair.address
  );
  const registrarIdentityTx = await setIdentity({
    accountPair: registrarAccountPair,
    api,
    nonce: registrarAccountNonce,
    info: registrarInfo
  });
  console.log(registrarIdentityTx);
  console.log("Set on-chain identity for user");
  const userInfo = {
    additional: [],
    display: { Raw: "0x4448525556" },
    legal: { None: null },
    web: { None: null },
    riot: { None: null },
    email: { None: null },
    pgpFingerprint: null,
    image: { None: null },
    twitter: { None: null }
  };
  userAccountNonce = await api.query.system.accountNonce(
    userAccountPair.address
  );
  const userIdentityTx = await setIdentity({
    accountPair: userAccountPair,
    api,
    nonce: userAccountNonce,
    info: userInfo
  });
  console.log(userIdentityTx);
  console.log("Make a sudo call to add registrar");
  sudoAccountNonce = await api.query.system.accountNonce(
    sudoAccountPair.address
  );
  const addRegistrarTx = await addRegistar({
    api,
    nonce: sudoAccountNonce,
    accountPair: sudoAccountPair,
    registrarAccountPair
  });
  console.log(addRegistrarTx);
  console.log("User is requesting judgement to the registrar");
  userAccountNonce = await api.query.system.accountNonce(
    userAccountPair.address
  );
  const requestJudgementTx = await requestJudgement({
    api,
    nonce: userAccountNonce,
    accountPair: userAccountPair,
    reg_index : 0,
    max_fee : 10
  });
  console.log(requestJudgementTx);
  console.log("Registrar is providing judgement to the user");
   registrarAccountNonce = await api.query.system.accountNonce(
    registrarAccountPair.address
  );
  const provideJudgementTx = await provideJudgement({
    api,
    nonce: registrarAccountNonce,
    accountPair: registrarAccountPair,
    reg_index : 0,
    userAccountPair,
    judgement : "feepaid" // unknown, feepaid, reasonable, knowngood, outofdate, lowquality, erroneous
  });
  console.log(provideJudgementTx);
  process.exit(0);
}

main().catch(console.error);
