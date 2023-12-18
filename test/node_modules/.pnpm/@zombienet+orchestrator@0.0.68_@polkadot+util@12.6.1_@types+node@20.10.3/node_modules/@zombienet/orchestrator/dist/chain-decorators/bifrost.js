"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getNodeKey = void 0;
function getNodeKey(node, useStash = true) {
    const { sr_stash, sr_account } = node.accounts;
    const address = useStash ? sr_stash.address : sr_account.address;
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
exports.default = {
    getNodeKey,
};
