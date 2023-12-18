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
exports.paraIsRegistered = exports.paraGetBlockHeight = void 0;
const debug = require("debug")("zombie::js-helpers::parachain");
function paraGetBlockHeight(api, paraId) {
    return __awaiter(this, void 0, void 0, function* () {
        const optHeadData = yield api.query.paras.heads(paraId);
        if (optHeadData === null || optHeadData === void 0 ? void 0 : optHeadData.isSome) {
            const header = api.createType("Header", optHeadData.unwrap().toHex());
            const headerStr = JSON.stringify(header === null || header === void 0 ? void 0 : header.toHuman(), null, 2);
            const headerObj = JSON.parse(headerStr);
            const blockNumber = parseInt(headerObj["number"].replace(",", ""));
            debug(`blockNumber : ${blockNumber}`);
            return blockNumber;
        }
        else {
            return 0;
        }
    });
}
exports.paraGetBlockHeight = paraGetBlockHeight;
function paraIsRegistered(api, paraId) {
    return __awaiter(this, void 0, void 0, function* () {
        const parachains = (yield api.query.paras.parachains()) || [];
        debug(`parachains : ${JSON.stringify(parachains)}`);
        const isRegistered = parachains.findIndex((id) => id.toString() == paraId.toString()) >= 0;
        return isRegistered;
    });
}
exports.paraIsRegistered = paraIsRegistered;
