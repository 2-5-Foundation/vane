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
exports.GrafanaResource = void 0;
const utils_1 = require("@zombienet/utils");
const promises_1 = __importDefault(require("fs/promises"));
const path_1 = __importDefault(require("path"));
class GrafanaResource {
    constructor(client, namespace, prometheusIp, tempoIp) {
        this.namespace = namespace;
        this.prometheusIp = prometheusIp;
        this.tempoIp = tempoIp;
        this.dataSourcesPath = `${client.tmpDir}/grafana/datasources`;
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
                yield (0, utils_1.makeDir)(this.dataSourcesPath, true);
            }
            catch (_a) {
                throw new Error("Error creating directory for grafana resource");
            }
        });
    }
    generateGrafanaConfig() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const templateConfigPath = path_1.default.resolve(__dirname, "./configs/grafana.yml");
                const grafanaConfigBuffer = yield promises_1.default.readFile(templateConfigPath);
                let grafanaConfig = grafanaConfigBuffer.toString("utf8");
                grafanaConfig = grafanaConfig
                    .replace("{{PROMETHEUS_IP}}", this.prometheusIp)
                    .replace("{{TEMPO_IP}}", this.tempoIp);
                yield promises_1.default.writeFile(`${this.dataSourcesPath}/prometheus.yml`, grafanaConfig);
            }
            catch (err) {
                console.error(utils_1.decorators.red("Error generating config for grafana resource"));
                throw err;
            }
        });
    }
    generateVolumes() {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.createVolumeDirectories();
            yield this.generateGrafanaConfig();
            return [
                {
                    name: "datasources-cfg",
                    hostPath: { type: "Directory", path: this.dataSourcesPath },
                },
            ];
        });
    }
    generateVolumesMounts() {
        return [
            {
                name: "datasources-cfg",
                mountPath: "/etc/grafana/provisioning/datasources",
                readOnly: false,
            },
        ];
    }
    generateContainersPorts() {
        return __awaiter(this, void 0, void 0, function* () {
            return [
                {
                    containerPort: 3000,
                    name: "grafana_web",
                    hostPort: yield (0, utils_1.getRandomPort)(),
                },
            ];
        });
    }
    generateContainers(volumeMounts, ports) {
        return [
            {
                image: "docker.io/grafana/grafana",
                name: "grafana",
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
                name: "grafana",
                namespace: this.namespace,
                labels: {
                    "zombie-role": "grafana",
                    app: "zombienet",
                    "zombie-ns": this.namespace,
                },
            },
            spec: {
                hostname: "grafana",
                restartPolicy: "OnFailure",
                volumes,
                containers,
            },
        };
    }
}
exports.GrafanaResource = GrafanaResource;
