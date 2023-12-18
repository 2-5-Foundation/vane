"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ServiceResource = void 0;
const constants_1 = require("../../../constants");
class ServiceResource {
    constructor(podSpec) {
        this.podSpec = podSpec;
    }
    generateSpec() {
        const ports = this.generatePorts();
        const name = this.podSpec.metadata.name;
        return this.generateServiceSpec(name, ports);
    }
    shouldExposeJaegerPorts() {
        return this.podSpec.spec.containers.some((container) => container.name === "jaeger-agent");
    }
    generatePorts() {
        let ports = [
            {
                name: "prometheus",
                protocol: "TCP",
                port: constants_1.PROMETHEUS_PORT,
                targetPort: constants_1.PROMETHEUS_PORT,
            },
            {
                name: "rpc-http",
                protocol: "TCP",
                port: constants_1.RPC_HTTP_PORT,
                targetPort: constants_1.RPC_HTTP_PORT,
            },
            {
                name: "rpc-ws",
                protocol: "TCP",
                port: constants_1.RPC_WS_PORT,
                targetPort: constants_1.RPC_WS_PORT,
            },
            {
                name: "p2p",
                protocol: "TCP",
                port: constants_1.P2P_PORT,
                targetPort: constants_1.P2P_PORT,
            },
        ];
        if (this.shouldExposeJaegerPorts()) {
            ports = ports.concat([
                {
                    name: "jaeger-agent-zipkin-compact",
                    protocol: "UDP",
                    port: constants_1.JAEGER_AGENT_ZIPKIN_COMPACT_PORT,
                    targetPort: constants_1.JAEGER_AGENT_ZIPKIN_COMPACT_PORT,
                },
                {
                    name: "jaeger-agent-serve-configs",
                    protocol: "TCP",
                    port: constants_1.JAEGER_AGENT_SERVE_CONFIGS_PORT,
                    targetPort: constants_1.JAEGER_AGENT_SERVE_CONFIGS_PORT,
                },
                {
                    name: "jaeger-agent-thrift-compact",
                    protocol: "UDP",
                    port: constants_1.JAEGER_AGENT_THRIFT_COMPACT_PORT,
                    targetPort: constants_1.JAEGER_AGENT_THRIFT_COMPACT_PORT,
                },
                {
                    name: "jaeger-agent-thrift-binary",
                    protocol: "UDP",
                    port: constants_1.JAEGER_AGENT_THRIFT_BINARY_PORT,
                    targetPort: constants_1.JAEGER_AGENT_THRIFT_BINARY_PORT,
                },
            ]);
        }
        return ports;
    }
    generateServiceSpec(name, ports) {
        return {
            apiVersion: "v1",
            kind: "Service",
            metadata: { name },
            spec: {
                selector: { "app.kubernetes.io/instance": name },
                ports,
            },
        };
    }
}
exports.ServiceResource = ServiceResource;
