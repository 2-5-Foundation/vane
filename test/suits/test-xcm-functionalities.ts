import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GAS_PRICE, customWeb3Request, generateKeyringPair } from "@moonwall/util";
import { ApiPromise, Keyring } from "@polkadot/api";
import "@polkadot/api-augment"


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
    }
})