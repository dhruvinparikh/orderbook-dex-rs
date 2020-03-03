const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { setSS58Format } = require("@polkadot/util-crypto");
const program = require("commander");
const fs = require("fs");
const BN = require("bn.js");
const { types } = require("./types");
const pkg = require("./package.json");

const MASTER_ACCOUNT_THRESHOLD = 100000;
const OTHER_ACCOUNT_THRESHOLD = 10000;
const DECIMALS = 4;

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

function restoreAccount(accountJson, password) {
    setSS58Format(42);
    const kr = new Keyring();
    const pair = kr.addFromJson(accountJson);
    pair.decodePkcs8(password);
    return pair;
}

function getAccountPairFromJSON(accountObj) {
    const account = JSON.parse(fs.readFileSync(Object.keys(accountObj)[0]));
    const accountPair = restoreAccount(
        account,
        accountObj[Object.keys(accountObj)[0]]
    );
    return accountPair;
}

async function getFreeBalance(api, address) {
    const balance = await api.query.balances.freeBalance(address);
    return balance;
}

async function getNonce(api, address) {
    const nonce = await api.query.system.accountNonce(address);
    return nonce;
}

async function sendTransaction(data) {
    const { to, amount, accountPair, api, nonce } = data;
    const promise = new Promise(async (resolve, reject) => {
        try {
            const unsub = await api.tx.balances
                .transfer(to, amount)
                .signAndSend(accountPair, { nonce }, ({ events = [], status }) => {
                    if (status.isFinalized) {
                        console.log(
                            `Transaction included at blockHash ${status.asFinalized}`
                        );
                        resolve({ status: "SUCCESS" });
                        unsub();
                    }
                    if (status.isDropped || status.isInvalid || status.isUsurped) {
                        console.log(`FAILURE`);
                        resolve({ status: "FAIL" });
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
            "--issuer <string>",
            "account object {<json-file-path1>:<password1>}"
        )
        .option(
            "--trader <string>",
            "account object {<json-file-path1>:<password1>}"
        );
    program.parse(process.argv);
    if (
        !program.url ||
        !program.masterAccount ||
        !program.issuer ||
        !program.trader
    ) {
        program.help();
    }
    return new Promise(resolve => {
        resolve({
            url: program.url,
            masterAccountObj: JSON.parse(program.masterAccount),
            issuerAccountObj: JSON.parse(program.issuer),
            traderAccountObj: JSON.parse(program.trader)
        });
    });
}

async function main() {
    const {
        url,
        masterAccountObj,
        issuerAccountObj,
        traderAccountObj
    } = await getArg();
    let api;
    let balance;
    try {
        api = await getApi(url);
    } catch (e) {
        console.log(`Error connecting to ${url}`);
        process.exit(1);
    }

    const masterAccountPair = getAccountPairFromJSON(masterAccountObj);
    const issuerAccountPair = getAccountPairFromJSON(issuerAccountObj);
    const traderAccountPair = getAccountPairFromJSON(traderAccountObj);

    console.log("Starting dex testing suite");

    console.log(
        `Fetching the balance of master account => ${masterAccountPair.address}`
    );
    balance = await getFreeBalance(api, masterAccountPair.address);
    console.log(
        `Balance of master account is ${balance
            .div(new BN(10).pow(new BN(DECIMALS)))
            .toString()} DNA/s`
    );
    if (!balance.gte(new BN(MASTER_ACCOUNT_THRESHOLD))) {
        console.log(`Master account does not have enough balance.`);
        process.exit(1);
    }

    console.log(
        `Fetching the balance of issuer => ${issuerAccountPair.address}`
    );
    balance = await getFreeBalance(api, issuerAccountPair.address);
    console.log(
        `Balance of issuer is ${balance
            .div(new BN(10).pow(new BN(DECIMALS)))
            .toString()} DNA/s`
    );
    if (!balance.gte(new BN(OTHER_ACCOUNT_THRESHOLD))) {
        console.log(
            `Funding issuer ${issuerAccountPair.address} from master account ${
            masterAccountPair.address
            } with amount : ${OTHER_ACCOUNT_THRESHOLD / 10000} DNAs`
        );
        const nonce = await getNonce(api, masterAccountPair.address);
        const tx = await sendTransaction({
            to: issuerAccountPair.address,
            amount: OTHER_ACCOUNT_THRESHOLD,
            accountPair: masterAccountPair,
            api,
            nonce
        });
        console.log(`Result : ${tx.status}`);
        balance = await getFreeBalance(api, issuerAccountPair.address);
        console.log(
            `Balance of issuer is ${balance
                .div(new BN(10).pow(new BN(DECIMALS)))
                .toString()} DNA/s`
        );
    }

    // issuer issue symbol
    let issuerNonce = await getNonce(api,issuerAccountPair.address);
    const [baseResult, quoteResult] = await Promise.all([
        new Promise(resolve => {
            api.tx.assets
                .issue("BTC", 2000000)
                .signAndSend(issuerAccountPair, { nonce:issuerNonce }, result => {
                    if (result.status.isFinalized) {
                        const record = result.findRecord("assets", "Issued");
                        if (record) {
                            resolve(record);
                        }
                    }
                });
        }),
        new Promise(resolve => {
            api.tx.assets
                .issue("ETH", 2000000)
                .signAndSend(issuerAccountPair, { nonce: issuerNonce.toNumber() + 1 }, result => {
                    if (result.status.isFinalized) {
                        const record = result.findRecord("assets", "Issued");
                        if (record) {
                            resolve(record);
                        }
                    }
                });
        })
    ]);
    let baseEvent = baseResult.toJSON().event.data; // Issued(AccountId, Hash, Balance)
    let quoteEvent = quoteResult.toJSON().event.data;
    console.log("Issuer issue base event: ", baseEvent);
    console.log("Issuer issue quote event: ", quoteEvent);

    console.log(
        `Fetching the balance of trader => ${traderAccountPair.address}`
    );
    balance = await getFreeBalance(api, traderAccountPair.address);
    console.log(
        `Balance of trader is ${balance
            .div(new BN(10).pow(new BN(DECIMALS)))
            .toString()} DNA/s`
    );
    if (!balance.gte(new BN(OTHER_ACCOUNT_THRESHOLD))) {
        console.log(
            `Funding trader ${traderAccountPair.address} from master account ${
            masterAccountPair.address
            } with amount : ${OTHER_ACCOUNT_THRESHOLD / 10000} DNAs`
        );
        const nonce = await getNonce(api, masterAccountPair.address);
        const tx = await sendTransaction({
            to: traderAccountPair.address,
            amount: OTHER_ACCOUNT_THRESHOLD,
            accountPair: masterAccountPair,
            api,
            nonce
        });
        console.log(`Result : ${tx.status}`);
        balance = await getFreeBalance(api, traderAccountPair.address);
        console.log(
            `Balance of trader is ${balance
                .div(new BN(10).pow(new BN(DECIMALS)))
                .toString()} DNA/s`
        );
    }

    // Issuer transfer to trader
    const transferRecord = await new Promise(resolve => {
        // Transferd(AccountId, AccountId, Hash, Balance)
        api.tx.assets
            .deposit(baseEvent[1],traderAccountPair.address, 600000)
            .signAndSend(issuerAccountPair, result => {
                if (result.status.isFinalized) {
                    const record = result.findRecord("assets", "Transfered");
                    if (record) {
                        resolve(record);
                    }
                }
            });
    });
    const transferEvent = transferRecord.toJSON().event.data;
    console.log("Issuer transfer to trader event: ", transferEvent);

    // Issuer create exchangePair
    const pairRecord = await new Promise(resolve => {
        // ExchangePairCreated(AccountId, Hash, ExchangePair),
        api.tx.dex
            .createExchangePair(baseEvent[1], quoteEvent[1])
            .signAndSend(issuerAccountPair, result => {
                if (result.status.isFinalized) {
                    const record = result.findRecord("dex", "ExchangePairCreated");
                    if (record) {
                        resolve(record);
                    }
                }
            });
    });
    const pairEvent = pairRecord.toJSON().event.data;
    console.log("Issuer create exchangePair event: ", pairEvent);

    // Issuer create limit order
    const orderRecord = await new Promise(resolve => {
        // OrderCreated (accountId, baseAssetHash, quoteAssetHash, orderHash, LimitOrder)
        api.tx.dex
            .createOrder(baseEvent[1], quoteEvent[1], 1, 1, 300000)
            .signAndSend(issuerAccountPair, result => {
                if (result.status.isFinalized) {
                    const record = result.findRecord("dex", "OrderCreated");
                    if (record) {
                        resolve(record);
                    }
                }
            });
    });

    const orderEvent = orderRecord.toJSON().event.data;
    console.log("Issuer create limit order event: ", orderEvent);

    // Trader create limit order
    let records = [];
    const order2Record = await new Promise(resolve => {
        // ExchangeCreated (accountId, baseAssetHash, quoteAssetHash, dexHash, Dex)
        api.tx.dex
            .createOrder(baseEvent[1], quoteEvent[1], 0, 1, 300000)
            .signAndSend(traderAccountPair, result => {
                if (result.status.isFinalized) {
                    const record1 = result.findRecord("dex", "OrderCreated");
                    const record2 = result.findRecord("dex", "ExchangeCreated");
                    if (record1) {
                        records.push(record1);
                    }
                    if (record2) {
                        records.push(record2);
                    }
                    if (records.length == 2) {
                        resolve(records);
                    }
                }
            });
    });
    const order2Event = order2Record[0].toJSON().event.data;
    const dexEvent = order2Record[1].toJSON().event.data;
    console.log("Trader create limit order event: ", order2Event);
    console.log("DEX created event: ", dexEvent);
}

main()
    .catch(console.error)
    .finally(() => process.exit());
