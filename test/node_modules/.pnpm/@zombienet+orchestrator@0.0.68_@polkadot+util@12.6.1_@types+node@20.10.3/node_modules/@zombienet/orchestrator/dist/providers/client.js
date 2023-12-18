"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.setClient = exports.getClient = exports.Client = void 0;
class Client {
    constructor(configPath, namespace, tmpDir, command, providerName) {
        this.podMonitorAvailable = false;
        this.configPath = configPath;
        this.namespace = namespace;
        this.debug = true;
        this.timeout = 30; // secs
        this.tmpDir = tmpDir;
        this.localMagicFilepath = `${tmpDir}/finished.txt`;
        this.command = command;
        this.providerName = providerName;
    }
}
exports.Client = Client;
let client;
function getClient() {
    if (!client)
        throw new Error("Client not initialized");
    return client;
}
exports.getClient = getClient;
function setClient(c) {
    if (client)
        throw new Error("Client already initialized");
    client = c;
}
exports.setClient = setClient;
