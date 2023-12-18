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
exports.spawnIntrospector = void 0;
const constants_1 = require("../constants");
const networkNode_1 = require("../networkNode");
function spawnIntrospector(client, node, inCI = false) {
    return __awaiter(this, void 0, void 0, function* () {
        const [nodeIp, port] = yield client.getNodeInfo(node.name, constants_1.RPC_HTTP_PORT);
        const wsUri = constants_1.WS_URI_PATTERN.replace("{{IP}}", nodeIp).replace("{{PORT}}", port.toString());
        yield client.spawnIntrospector(wsUri);
        const IP = inCI ? yield client.getNodeIP(constants_1.INTROSPECTOR_POD_NAME) : constants_1.LOCALHOST;
        const PORT = inCI
            ? constants_1.INTROSPECTOR_PORT
            : yield client.startPortForwarding(constants_1.INTROSPECTOR_PORT, constants_1.INTROSPECTOR_POD_NAME);
        // TODO: create a new kind `companion`
        return new networkNode_1.NetworkNode(constants_1.INTROSPECTOR_POD_NAME, "", constants_1.METRICS_URI_PATTERN.replace("{{IP}}", IP).replace("{{PORT}}", PORT.toString()), "");
    });
}
exports.spawnIntrospector = spawnIntrospector;
