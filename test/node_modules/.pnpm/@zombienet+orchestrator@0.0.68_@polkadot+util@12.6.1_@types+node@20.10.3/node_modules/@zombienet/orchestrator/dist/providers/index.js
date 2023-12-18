"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getProvider = exports.Providers = void 0;
const k8s_1 = require("./k8s");
const native_1 = require("./native");
const podman_1 = require("./podman");
exports.Providers = new Map();
exports.Providers.set("kubernetes", k8s_1.provider);
exports.Providers.set("podman", podman_1.provider);
exports.Providers.set("native", native_1.provider);
function getProvider(provider) {
    if (!exports.Providers.has(provider)) {
        throw new Error("Invalid provider config. You must one of: " +
            Array.from(exports.Providers.keys()).join(", "));
    }
    return exports.Providers.get(provider);
}
exports.getProvider = getProvider;
