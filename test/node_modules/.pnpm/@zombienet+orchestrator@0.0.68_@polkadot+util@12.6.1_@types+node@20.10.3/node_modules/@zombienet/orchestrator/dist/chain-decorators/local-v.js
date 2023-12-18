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
exports.addCollatorSelection = exports.getNodeKey = void 0;
const api_1 = require("@polkadot/api");
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
const utils_1 = require("@zombienet/utils");
const chainSpec_1 = require("../chainSpec");
const keys_1 = require("../keys");
function generateKeyForNode(nodeName) {
    return __awaiter(this, void 0, void 0, function* () {
        const keys = yield (0, keys_1.generateKeyForNode)(nodeName);
        yield (0, util_crypto_1.cryptoWaitReady)();
        const eth_keyring = new api_1.Keyring({ type: "ethereum" });
        const eth_account = eth_keyring.createFromUri(`${keys.mnemonic}/m/44'/60'/0'/0/0`);
        keys.eth_account = {
            address: eth_account.address,
            publicKey: (0, util_1.u8aToHex)(eth_account.publicKey),
        };
        return keys;
    });
}
function getNodeKey(node) {
    try {
        const { sr_account, eth_account } = node.accounts;
        const key = [
            eth_account.address,
            eth_account.address,
            {
                aura: sr_account.address,
            },
        ];
        return key;
    }
    catch (err) {
        console.error(`\n${utils_1.decorators.red(`Fail to generate key for node: ${node}`)}`);
        throw err;
    }
}
exports.getNodeKey = getNodeKey;
function addCollatorSelection(specPath, node) {
    var _a;
    return __awaiter(this, void 0, void 0, function* () {
        try {
            const chainSpec = (0, chainSpec_1.readAndParseChainSpec)(specPath);
            const runtimeConfig = (0, chainSpec_1.getRuntimeConfig)(chainSpec);
            if (!((_a = runtimeConfig === null || runtimeConfig === void 0 ? void 0 : runtimeConfig.collatorSelection) === null || _a === void 0 ? void 0 : _a.invulnerables))
                return;
            const { eth_account } = node.accounts;
            runtimeConfig.collatorSelection.invulnerables.push(eth_account.address);
            new utils_1.CreateLogTable({
                colWidths: [30, 20, 70],
            }).pushToPrint([
                [
                    utils_1.decorators.cyan("👤 Added CollatorSelection "),
                    utils_1.decorators.green(node.name),
                    utils_1.decorators.magenta(eth_account.address),
                ],
            ]);
            (0, chainSpec_1.writeChainSpec)(specPath, chainSpec);
        }
        catch (err) {
            console.error(`\n${utils_1.decorators.red(`Fail to add collator: ${node}`)}`);
            throw err;
        }
    });
}
exports.addCollatorSelection = addCollatorSelection;
exports.default = {
    getNodeKey,
    generateKeyForNode,
    addCollatorSelection,
};
