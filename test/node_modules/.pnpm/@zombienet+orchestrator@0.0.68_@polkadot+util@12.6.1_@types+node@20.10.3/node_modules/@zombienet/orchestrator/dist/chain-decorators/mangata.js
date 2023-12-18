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
exports.getNodeKey = void 0;
const chainSpec_1 = require("../chainSpec");
// Track 1st staking bond as default
let paraStakingBond;
function getNodeKey(node) {
    const { sr_account } = node.accounts;
    const address = sr_account.address;
    const key = [
        address,
        address,
        {
            aura: address,
        },
    ];
    return key;
}
exports.getNodeKey = getNodeKey;
function clearAuthorities(specPath) {
    return __awaiter(this, void 0, void 0, function* () {
        yield (0, chainSpec_1.clearAuthorities)(specPath);
        const chainSpec = (0, chainSpec_1.readAndParseChainSpec)(specPath);
        const runtimeConfig = (0, chainSpec_1.getRuntimeConfig)(chainSpec);
        // Clear parachainStaking candidates
        if (runtimeConfig === null || runtimeConfig === void 0 ? void 0 : runtimeConfig.parachainStaking) {
            paraStakingBond = runtimeConfig.parachainStaking.candidates[0][1];
            runtimeConfig.parachainStaking.candidates.length = 0;
            runtimeConfig.parachainStaking.delegations.length = 0;
        }
        (0, chainSpec_1.writeChainSpec)(specPath, chainSpec);
    });
}
function addParaCustom(specPath, node) {
    return __awaiter(this, void 0, void 0, function* () {
        const chainSpec = (0, chainSpec_1.readAndParseChainSpec)(specPath);
        const runtimeConfig = (0, chainSpec_1.getRuntimeConfig)(chainSpec);
        if (!(runtimeConfig === null || runtimeConfig === void 0 ? void 0 : runtimeConfig.parachainStaking))
            return;
        const { sr_account } = node.accounts;
        const stakingBond = paraStakingBond || 1000000000000;
        // Ensure collator account has enough balance to bond and add candidate
        runtimeConfig.tokens.tokensEndowment.push([
            sr_account.address,
            0,
            stakingBond,
        ]);
        runtimeConfig.parachainStaking.candidates.push([
            sr_account.address,
            stakingBond,
            0,
        ]);
        (0, chainSpec_1.writeChainSpec)(specPath, chainSpec);
    });
}
exports.default = {
    getNodeKey,
    clearAuthorities,
    addParaCustom,
};
