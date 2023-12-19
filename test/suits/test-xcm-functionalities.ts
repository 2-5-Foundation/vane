import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { KeyringPair, MIN_GAS_PRICE, customWeb3Request, generateKeyringPair } from "@moonwall/util";
import { ApiPromise, Keyring } from "@polkadot/api";
import "@polkadot/api-augment"
import "@polkadot/api-augment/polkadot"
import { nToBigInt, stringToU8a } from "@polkadot/util";

function signAndSendAndInclude(tx, account): Promise<{ txHash; blockHash; status }> {
    return new Promise((resolve) => {
        tx.signAndSend(account, ({ status, txHash }) => {
            if (status.isInBlock) {
                resolve({
                    txHash,
                    blockHash: status.asInBlock,
                    status,
                });
            }
        });
    });
}

describeSuite({
    id: "P01",
    title: "Vane Para",
    foundationMethods: "zombie",

    testCases: function({it, context}) {
        let VaneParaApi: ApiPromise;
        let RococoApi: ApiPromise;

        beforeAll(async ()=> {
            // Pre checks
            VaneParaApi = context.polkadotJs("Vane_Para");
            RococoApi = context.polkadotJs("Rococo");

            const relayNetwork = RococoApi.consts.system.version.specName.toString();
            expect(relayNetwork, "Relay API incorrect").to.contain("rococo");

            const paraNetwork = VaneParaApi.consts.system.version.specName.toString();
            const paraId1000 = (await VaneParaApi.query.parachainInfo.parachainId()).toString();
            expect(paraNetwork, "Para API incorrect").to.contain("vane-parachain");
            expect(paraId1000, "Para API incorrect").to.be.equal("2000");


        },120000);

        
        it({
            id: "T01",
            title: "Blocks are being produced on parachain",
            test: async function () {
                const blockNum = (await VaneParaApi.rpc.chain.getBlock()).block.header.number.toNumber();
                expect(blockNum).to.be.greaterThan(0);
            },
        });

        
        it({
            id:"T02",
            title: "Check reserveTransfer from Xcm Pallet from Rococo Relay",
            test:async () => {
                const keyring = new Keyring({ type: "sr25519" });

                const alice:KeyringPair = keyring.addFromUri("//Alice", { name: "Alice default" });
                const bob:KeyringPair = keyring.addFromUri("//Bob", { name: "Bob default" });
                const vane_account = '5Ec4AhPUwPeyTFyuhGuBbD224mY85LKLMSqSSo33JYWCazU4';


                console.log("Alice Sender: \n")
                console.log(alice.address)

                console.log("Bob Receiver: \n")
                console.log(bob.address)

                let aliceId = stringToU8a(alice.address);
                
                let amount:BigInt = nToBigInt(100000) * nToBigInt(1000000000000);
              
              
                let dest = {
                  V3: {
                    parents: 0,
                    interior: {
                      X1: {
                        parachain: 2000
                      }
                    }
                  } 
                };
              
                let beneficiary = {
                  V3: {
                    parents: 0,
                    interior: {
                      X1: {
                        accountid32: {
                          network: null,
                          id: aliceId
                        }
                      }
                    }
                  } 
                };
              
                let asset = {
                  V3 : [
                    {
                      id: {
                        'Concrete' : {
                          parents: 0,
                          interior: 'Here'
                        }
                      },
              
                      fun: {
                        'Fungible': amount
                      }
                      
                    }
                  ]
                };

                // check initial balance
                const data = await RococoApi.query.system.account(vane_account);
                console.log("Initial Balance: ")
                console.log(data.data.free.toString())
              
               const xcmReserveCall = RococoApi.tx.xcmPallet.reserveTransferAssets(
                    dest,
                    beneficiary,
                    asset,
                    0
               );

               await signAndSendAndInclude(xcmReserveCall,alice)

                // Assert vane derived account balance
                const { nonce, data: balance } = await RococoApi.query.system.account(vane_account);
                console.log(balance.free.toString())
                expect(balance.free.toString()).to.be.equal("100000000000000000");

                // Check event in vane parachain
               // no blockHash is specified, so we retrieve the latest
                const signedBlock = await VaneParaApi.rpc.chain.getBlock();

                // get the api and events at a specific block
                const apiAt = await VaneParaApi.at(signedBlock.block.header.hash);
                const allRecords = await apiAt.query.system.events();

                // map between the extrinsics and events
                signedBlock.block.extrinsics.forEach(({ method: { method, section } }, index) => {
                allRecords
                    // filter the specific events based on the phase and then the
                    // index of our extrinsic in the block
                    .filter(({ phase }) =>
                    phase.isApplyExtrinsic &&
                    phase.asApplyExtrinsic.eq(index)
                    )
                    // test the events against the specific types we are looking for
                    .forEach(({ event }) => {
                    if (VaneParaApi.events.system.ExtrinsicSuccess.is(event)) {
                        // extract the data for this event
                        // (In TS, because of the guard above, these will be typed)
                        const [dispatchInfo] = event.data;

                        console.log(`${section}.${method}:: ExtrinsicSuccess:: ${JSON.stringify(dispatchInfo.toHuman())}`);
                    } else if (VaneParaApi.events.system.ExtrinsicFailed.is(event)) {
                        // extract the data for this event
                        const [dispatchError, dispatchInfo] = event.data;
                        let errorInfo;

                        // decode the error
                        if (dispatchError.isModule) {
                        // for module errors, we have the section indexed, lookup
                        // (For specific known errors, we can also do a check against the
                        // api.errors.<module>.<ErrorName>.is(dispatchError.asModule) guard)
                        const decoded = VaneParaApi.registry.findMetaError(dispatchError.asModule);

                        errorInfo = `${decoded.section}.${decoded.name}`;
                        } else {
                        // Other, CannotLookup, BadOrigin, no extra info
                        errorInfo = dispatchError.toString();
                        }

                        console.log(`${section}.${method}:: ExtrinsicFailed:: ${errorInfo}`);
                    }
                    });
                });
                
                // Check DOT balance in Alice in vane parachain
                const dotBalance = await VaneParaApi.query.vaneAssets.account("DOT",alice.address);
                

            }
        })
    }
})