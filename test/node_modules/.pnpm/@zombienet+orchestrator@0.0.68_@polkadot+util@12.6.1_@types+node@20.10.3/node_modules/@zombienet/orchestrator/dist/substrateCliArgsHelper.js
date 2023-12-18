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
exports.setSubstrateCliArgsVersion = void 0;
const utils_1 = require("@zombienet/utils");
const providers_1 = require("./providers");
const setSubstrateCliArgsVersion = (network, client) => __awaiter(void 0, void 0, void 0, function* () {
    const { getCliArgsVersion } = (0, providers_1.getProvider)(client.providerName);
    // Calculate substrate cli version for each node
    // and set in the node to use later when we build the cmd.
    const imgCmdMap = new Map();
    network.relaychain.nodes.reduce((memo, node) => {
        if (node.substrateCliArgsVersion)
            return memo;
        const uniq_image_cmd = `${node.image}_${node.command}`;
        if (!memo.has(uniq_image_cmd))
            memo.set(uniq_image_cmd, { image: node.image, command: node.command });
        return memo;
    }, imgCmdMap);
    network.parachains.reduce((memo, parachain) => {
        for (const collator of parachain.collators) {
            if (collator.substrateCliArgsVersion)
                return memo;
            const uniq_image_cmd = `${collator.image}_${collator.command}`;
            if (!memo.has(uniq_image_cmd))
                memo.set(uniq_image_cmd, {
                    image: collator.image,
                    command: collator.command,
                });
        }
        return memo;
    }, imgCmdMap);
    // check versions in series
    const promiseGenerators = [];
    for (const [, v] of imgCmdMap) {
        const getVersionPromise = () => __awaiter(void 0, void 0, void 0, function* () {
            const version = yield getCliArgsVersion(v.image, v.command);
            v.version = version;
            return version;
        });
        promiseGenerators.push(getVersionPromise);
    }
    yield (0, utils_1.series)(promiseGenerators, 4);
    // now we need to iterate and set in each node the version
    // IFF is not set
    for (const node of network.relaychain.nodes) {
        if (node.substrateCliArgsVersion)
            continue;
        const uniq_image_cmd = `${node.image}_${node.command}`;
        node.substrateCliArgsVersion = imgCmdMap.get(uniq_image_cmd).version;
    }
    for (const parachain of network.parachains) {
        for (const collator of parachain.collators) {
            if (collator.substrateCliArgsVersion)
                continue;
            const uniq_image_cmd = `${collator.image}_${collator.command}`;
            collator.substrateCliArgsVersion = imgCmdMap.get(uniq_image_cmd).version;
        }
    }
});
exports.setSubstrateCliArgsVersion = setSubstrateCliArgsVersion;
