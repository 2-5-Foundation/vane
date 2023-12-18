"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateRuntimeCode = exports.registerParachain = exports.paraIsRegistered = exports.paraGetBlockHeight = exports.findPatternInSystemEventSubscription = exports.connect = exports.chainUpgradeFromUrl = exports.chainUpgradeFromLocalFile = exports.chainCustomSectionUpgrade = void 0;
const api_1 = require("@polkadot/api");
const keyring_1 = require("@polkadot/keyring");
const util_crypto_1 = require("@polkadot/util-crypto");
const utils_1 = require("@zombienet/utils");
const chainUpgrade_1 = require("./chainUpgrade");
Object.defineProperty(exports, "chainCustomSectionUpgrade", { enumerable: true, get: function () { return chainUpgrade_1.chainCustomSectionUpgrade; } });
Object.defineProperty(exports, "chainUpgradeFromLocalFile", { enumerable: true, get: function () { return chainUpgrade_1.chainUpgradeFromLocalFile; } });
Object.defineProperty(exports, "chainUpgradeFromUrl", { enumerable: true, get: function () { return chainUpgrade_1.chainUpgradeFromUrl; } });
Object.defineProperty(exports, "validateRuntimeCode", { enumerable: true, get: function () { return chainUpgrade_1.validateRuntimeCode; } });
const events_1 = require("./events");
Object.defineProperty(exports, "findPatternInSystemEventSubscription", { enumerable: true, get: function () { return events_1.findPatternInSystemEventSubscription; } });
const parachain_1 = require("./parachain");
Object.defineProperty(exports, "paraGetBlockHeight", { enumerable: true, get: function () { return parachain_1.paraGetBlockHeight; } });
Object.defineProperty(exports, "paraIsRegistered", { enumerable: true, get: function () { return parachain_1.paraIsRegistered; } });
function connect(apiUrl, types) {
    return __awaiter(this, void 0, void 0, function* () {
        const provider = new api_1.WsProvider(apiUrl);
        const api = new api_1.ApiPromise({ provider, types });
        yield api.isReady;
        return api;
    });
}
exports.connect = connect;
function registerParachain({ id, wasmPath, statePath, apiUrl, onboardAsParachain, seed = "//Alice", finalization = false, }) {
    return __awaiter(this, void 0, void 0, function* () {
        return new Promise((resolve, reject) => __awaiter(this, void 0, void 0, function* () {
            yield (0, util_crypto_1.cryptoWaitReady)();
            const keyring = new keyring_1.Keyring({ type: "sr25519" });
            const sudo = keyring.addFromUri(seed);
            const api = yield connect(apiUrl);
            let nonce = (yield api.query.system.account(sudo.address)).nonce.toNumber();
            const wasm_data = (0, utils_1.readDataFile)(wasmPath);
            const genesis_state = (0, utils_1.readDataFile)(statePath);
            const parachainGenesisArgs = {
                genesis_head: genesis_state,
                validation_code: wasm_data,
                parachain: onboardAsParachain,
            };
            const genesis = api.createType("ParaGenesisArgs", parachainGenesisArgs);
            console.log(`Submitting extrinsic to register parachain ${id}. nonce: ${nonce}`);
            const unsub = yield api.tx.sudo
                .sudo(api.tx.parasSudoWrapper.sudoScheduleParaInitialize(id, genesis))
                .signAndSend(sudo, { nonce: nonce, era: 0 }, (result) => {
                console.log(`Current status is ${result.status}`);
                if (result.status.isInBlock) {
                    console.log(`Transaction included at blockhash ${result.status.asInBlock}`);
                    if (finalization) {
                        console.log("Waiting for finalization...");
                    }
                    else {
                        unsub();
                        return resolve();
                    }
                }
                else if (result.status.isFinalized) {
                    console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
                    unsub();
                    return resolve();
                }
                else if (result.isError) {
                    console.log(`Transaction error`);
                    reject(`Transaction error`);
                }
            });
            nonce += 1;
        }));
    });
}
exports.registerParachain = registerParachain;
