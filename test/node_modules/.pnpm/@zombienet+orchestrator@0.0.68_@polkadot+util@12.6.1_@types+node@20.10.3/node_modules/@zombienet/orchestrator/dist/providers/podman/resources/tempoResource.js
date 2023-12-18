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
exports.TempoResource = void 0;
const utils_1 = require("@zombienet/utils");
const promises_1 = __importDefault(require("fs/promises"));
const path_1 = __importDefault(require("path"));
class TempoResource {
    constructor(client, namespace) {
        this.namespace = namespace;
        const nodeRootPath = `${client.tmpDir}/tempo`;
        this.configPath = `${nodeRootPath}/etc`;
        this.dataPath = `${nodeRootPath}/data`;
    }
    generateSpec() {
        return __awaiter(this, void 0, void 0, function* () {
            const volumes = yield this.generateVolumes();
            const volumeMounts = this.generateVolumesMounts();
            const containersPorts = yield this.generateContainersPorts();
            const containers = this.generateContainers(volumeMounts, containersPorts);
            return this.generatePodSpec(containers, volumes);
        });
    }
    createVolumeDirectories() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                yield (0, utils_1.makeDir)(this.configPath, true);
                yield (0, utils_1.makeDir)(this.dataPath, true);
            }
            catch (_a) {
                throw new Error("Error creating directories for tempo resource");
            }
        });
    }
    generateTempoConfig() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const templateConfigPath = path_1.default.resolve(__dirname, `./configs/tempo.yaml`);
                yield promises_1.default.copyFile(templateConfigPath, `${this.configPath}/tempo.yaml`);
            }
            catch (_a) {
                throw new Error("Error generating config for tempo resource");
            }
        });
    }
    generateVolumes() {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.createVolumeDirectories();
            yield this.generateTempoConfig();
            return [
                {
                    name: "tempo-cfg",
                    hostPath: { type: "Directory", path: this.configPath },
                },
                {
                    name: "tempo-data",
                    hostPath: { type: "Directory", path: this.dataPath },
                },
            ];
        });
    }
    generateVolumesMounts() {
        return [
            {
                name: "tempo-cfg",
                mountPath: "/etc/tempo",
                readOnly: false,
            },
            {
                name: "tempo-data",
                mountPath: "/data",
                readOnly: false,
            },
        ];
    }
    generateContainersPorts() {
        return __awaiter(this, void 0, void 0, function* () {
            return [
                {
                    containerPort: 3100,
                    name: "tempo",
                    hostPort: yield (0, utils_1.getRandomPort)(),
                },
                {
                    containerPort: 14268,
                    name: "jaeger_ingest",
                    hostPort: yield (0, utils_1.getRandomPort)(),
                },
                {
                    containerPort: 4317,
                    name: "otlp_grpc",
                    hostPort: yield (0, utils_1.getRandomPort)(),
                },
                {
                    containerPort: 4318,
                    name: "otlp_http",
                    hostPort: yield (0, utils_1.getRandomPort)(),
                },
                {
                    containerPort: 9411,
                    name: "zipkin",
                    hostPort: yield (0, utils_1.getRandomPort)(),
                },
            ];
        });
    }
    generateContainers(volumeMounts, ports) {
        return [
            {
                image: "docker.io/grafana/tempo:latest",
                name: "tempo",
                args: ["-config.file=/etc/tempo/tempo.yaml"],
                imagePullPolicy: "Always",
                ports,
                volumeMounts,
            },
        ];
    }
    generatePodSpec(containers, volumes) {
        return {
            apiVersion: "v1",
            kind: "Pod",
            metadata: {
                name: "tempo",
                namespace: this.namespace,
                labels: {
                    "zombie-role": "tempo",
                    app: "zombienet",
                    "zombie-ns": this.namespace,
                },
            },
            spec: {
                hostname: "tempo",
                restartPolicy: "OnFailure",
                volumes,
                containers,
            },
        };
    }
}
exports.TempoResource = TempoResource;
