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
exports.IntrospectorResource = void 0;
const utils_1 = require("@zombienet/utils");
const constants_1 = require("../../../constants");
class IntrospectorResource {
    constructor(namespace, wsUri) {
        this.namespace = namespace;
        this.wsUri = wsUri;
    }
    generateSpec() {
        return __awaiter(this, void 0, void 0, function* () {
            const containerPorts = yield this.generateContainersPorts();
            const containers = this.generateContainers(containerPorts);
            return this.generatePodSpec(containers);
        });
    }
    generateContainersPorts() {
        return __awaiter(this, void 0, void 0, function* () {
            return [
                {
                    containerPort: 65432,
                    name: "prometheus",
                    hostPort: yield (0, utils_1.getRandomPort)(),
                },
            ];
        });
    }
    generateContainers(ports) {
        return [
            {
                image: "docker.io/paritytech/polkadot-introspector:latest",
                name: constants_1.INTROSPECTOR_POD_NAME,
                args: ["block-time-monitor", `--ws=${this.wsUri}`, "prometheus"],
                imagePullPolicy: "Always",
                ports,
                volumeMounts: [],
            },
        ];
    }
    generatePodSpec(containers) {
        return {
            apiVersion: "v1",
            kind: "Pod",
            metadata: {
                name: constants_1.INTROSPECTOR_POD_NAME,
                namespace: this.namespace,
                labels: {
                    "zombie-role": constants_1.INTROSPECTOR_POD_NAME,
                    app: "zombienet",
                    "zombie-ns": this.namespace,
                },
            },
            spec: {
                hostname: constants_1.INTROSPECTOR_POD_NAME,
                containers: containers,
                restartPolicy: "OnFailure",
            },
        };
    }
}
exports.IntrospectorResource = IntrospectorResource;
