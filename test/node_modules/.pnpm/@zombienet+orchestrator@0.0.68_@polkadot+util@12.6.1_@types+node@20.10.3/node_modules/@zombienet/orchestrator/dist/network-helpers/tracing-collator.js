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
exports.setTracingCollatorConfig = void 0;
const utils_1 = require("@zombienet/utils");
const constants_1 = require("../constants");
function setTracingCollatorConfig(networkSpec, network, client) {
    return __awaiter(this, void 0, void 0, function* () {
        const { tracing_collator_url, tracing_collator_service_port, tracing_collator_service_name, tracing_collator_service_namespace, } = networkSpec.settings;
        if (tracing_collator_url)
            network.tracing_collator_url = tracing_collator_url;
        else {
            const servicePort = tracing_collator_service_port || constants_1.TRACING_COLLATOR_PORT;
            switch (client.providerName) {
                case "kubernetes":
                    // check if we have the service available
                    const serviceName = tracing_collator_service_name || constants_1.TRACING_COLLATOR_SERVICE;
                    const serviceNamespace = tracing_collator_service_namespace || constants_1.TRACING_COLLATOR_NAMESPACE;
                    // check if service exists
                    let serviceExist;
                    try {
                        yield client.runCommand([
                            "get",
                            "service",
                            serviceName,
                            "-n",
                            serviceNamespace,
                        ]);
                        serviceExist = true;
                    }
                    catch (_) {
                        console.log(utils_1.decorators.yellow(`\n\t Warn: Tracing collator service doesn't exist`));
                    }
                    if (serviceExist) {
                        try {
                            const tracingPort = yield client.startPortForwarding(servicePort, `service/${serviceName}`, serviceNamespace);
                            network.tracing_collator_url = `http://localhost:${tracingPort}`;
                        }
                        catch (err) {
                            console.log(utils_1.decorators.yellow(`\n\t Warn: Can not create the forwarding to the tracing collator`));
                            console.error(err);
                        }
                    }
                    break;
                case "podman":
                    const tracingPort = yield client.getPortMapping(servicePort, constants_1.TRACING_COLLATOR_PODNAME);
                    network.tracing_collator_url = `http://localhost:${tracingPort}`;
                    break;
            }
        }
    });
}
exports.setTracingCollatorConfig = setTracingCollatorConfig;
