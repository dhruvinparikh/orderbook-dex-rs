const { ApiPromise,WsProvider, ApiRx } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');

const ACC_2_ED = '5GiFdKY55j8EwviqBVi2Jd6MNb6GnZDRZnc2tVSENmEdn1Vw';
const toMnemonics = "grace require cluster fringe insane pulse slab orient palm spot shaft soda";

async function main () {
  // Initialise the provider to connect to the local node
  const provider = new WsProvider('ws://127.0.0.1:9944');

  // Create the API and wait until ready
  const api = await ApiRx.create({provider}).toPromise();
  // Retrieve the chain & node information information via rpc calls
  const [chain, nodeName, nodeVersion] = await Promise.all([
    api.rpc.system.chain(),
    api.rpc.system.name(),
    api.rpc.system.version()
  ]);

  console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`);

  // Create an instance of the keyring
  const keyring = new Keyring({ type: 'sr25519' });
  // Add ACC2 to our keyring (with the known mnemonic for the account)
  const ACC2 = keyring.addFromMnemonic(toMnemonics);

  // Create a extrinsic, transferring 12345 units to ACC_2_SC.
  const subscription = api.tx.balances
    // create transfer
    .transfer(ACC_2_ED, 2000)
    // Sign and send the transcation
    .signAndSend(ACC2)
    // Subscribe to the status updates of the transfer
    .subscribe(({ status }) => {
      if (status.isFinalized) {
        console.log(`Successful transfer of 200 from ACC2 to ACC_2_ED with hash ${status.asFinalized.toHex()}`);
        subscription.unsubscribe();
      } else {
        console.log(`Status of transfer: ${status.type}`);
      }
    });
}

main().catch(console.error);