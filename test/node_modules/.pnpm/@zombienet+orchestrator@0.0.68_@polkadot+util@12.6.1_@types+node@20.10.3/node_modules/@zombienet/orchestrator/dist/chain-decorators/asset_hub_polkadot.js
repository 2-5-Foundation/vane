"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getNodeKey = void 0;
const chainSpec_1 = require("../chainSpec");
function getNodeKey(node, useStash = true) {
    const { ed_account } = node.accounts;
    const key = (0, chainSpec_1.getNodeKey)(node, useStash);
    key[2].aura = ed_account.address;
    return key;
}
exports.getNodeKey = getNodeKey;
exports.default = {
    getNodeKey,
};
