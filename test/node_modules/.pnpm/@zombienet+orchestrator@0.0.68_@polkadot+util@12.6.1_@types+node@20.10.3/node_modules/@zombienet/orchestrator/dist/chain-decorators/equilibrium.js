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
exports.clearAuthorities = exports.getNodeKey = exports.addAuthority = void 0;
const utils_1 = require("@zombienet/utils");
const chainSpec_1 = require("../chainSpec");
function addAuthority(specPath, node, key) {
    var _a, _b;
    return __awaiter(this, void 0, void 0, function* () {
        const chainSpec = (0, chainSpec_1.readAndParseChainSpec)(specPath);
        const { sr_stash } = node.accounts;
        const config = (0, chainSpec_1.getRuntimeConfig)(chainSpec);
        const keys = (_a = config.session) === null || _a === void 0 ? void 0 : _a.keys;
        if (!keys) {
            config.session = { keys: [] };
        }
        else {
            keys.push(key);
        }
        const eqKeys = (_b = config.eqSessionManager) === null || _b === void 0 ? void 0 : _b.validators;
        if (!eqKeys) {
            config.eqSessionManager = { validators: [key[0]] };
        }
        else {
            eqKeys.push(key[0]);
        }
        new utils_1.CreateLogTable({
            colWidths: [30, 20, 70],
        }).pushToPrint([
            [
                utils_1.decorators.cyan("👤 Added Genesis Authority"),
                utils_1.decorators.green(node.name),
                utils_1.decorators.magenta(sr_stash.address),
            ],
        ]);
        (0, chainSpec_1.writeChainSpec)(specPath, chainSpec);
    });
}
exports.addAuthority = addAuthority;
function getNodeKey(node, useStash = true) {
    const { sr_stash, sr_account, ed_account } = node.accounts;
    const address = useStash ? sr_stash.address : sr_account.address;
    const key = [
        address,
        address,
        {
            aura: sr_account.address,
            eq_rate: ed_account.address,
        },
    ];
    return key;
}
exports.getNodeKey = getNodeKey;
function clearAuthorities(specPath) {
    const chainSpec = (0, chainSpec_1.readAndParseChainSpec)(specPath);
    const runtimeConfig = (0, chainSpec_1.getRuntimeConfig)(chainSpec);
    // clear keys
    if (runtimeConfig === null || runtimeConfig === void 0 ? void 0 : runtimeConfig.session)
        runtimeConfig.session.keys.length = 0;
    // clear aura
    if (runtimeConfig === null || runtimeConfig === void 0 ? void 0 : runtimeConfig.aura)
        runtimeConfig.aura.authorities.length = 0;
    // clear grandpa
    if (runtimeConfig === null || runtimeConfig === void 0 ? void 0 : runtimeConfig.grandpa)
        runtimeConfig.grandpa.authorities.length = 0;
    // clear collatorSelection
    if (runtimeConfig === null || runtimeConfig === void 0 ? void 0 : runtimeConfig.collatorSelection)
        runtimeConfig.collatorSelection.invulnerables = [];
    // clear eqSession validators
    if (runtimeConfig === null || runtimeConfig === void 0 ? void 0 : runtimeConfig.eqSessionManager)
        runtimeConfig.eqSessionManager = { validators: [] };
    (0, chainSpec_1.writeChainSpec)(specPath, chainSpec);
    const logTable = new utils_1.CreateLogTable({
        colWidths: [120],
    });
    logTable.pushToPrint([
        [utils_1.decorators.green("🧹 Starting with a fresh authority set...")],
    ]);
}
exports.clearAuthorities = clearAuthorities;
exports.default = {
    getNodeKey,
    addAuthority,
    clearAuthorities,
};
