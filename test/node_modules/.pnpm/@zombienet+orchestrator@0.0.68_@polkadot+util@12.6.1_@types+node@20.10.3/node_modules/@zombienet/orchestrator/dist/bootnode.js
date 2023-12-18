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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.generateNodeMultiAddress = void 0;
const util_1 = require("@polkadot/util");
const libp2p_crypto_1 = require("libp2p-crypto");
const peer_id_1 = __importDefault(require("peer-id"));
function generateNodeMultiAddress(key, args, ip, port, useWs = true, certhash) {
    return __awaiter(this, void 0, void 0, function* () {
        let multiaddress;
        const pair = yield libp2p_crypto_1.keys.generateKeyPairFromSeed("Ed25519", (0, util_1.hexToU8a)((0, util_1.hexAddPrefix)(key)), 1024);
        const peerId = yield peer_id_1.default.createFromPrivKey(pair.bytes);
        const listenIndex = args.findIndex((arg) => arg === "--listen-addr");
        if (listenIndex >= 0) {
            const listenAddrParts = args[listenIndex + 1].split("/");
            listenAddrParts[2] = ip;
            listenAddrParts[4] = port.toString();
            if (certhash)
                listenAddrParts.push("certhash", certhash);
            multiaddress = `${listenAddrParts.join("/")}/p2p/${peerId.toB58String()}`;
        }
        else {
            multiaddress = `/ip4/${ip}/tcp/${port}/${useWs ? "ws/" : "/"}p2p/${peerId.toB58String()}`;
        }
        return multiaddress;
    });
}
exports.generateNodeMultiAddress = generateNodeMultiAddress;
