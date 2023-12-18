import { Client } from "../../client";
import { PodSpec } from "./types";
export declare class PrometheusResource {
    private readonly namespace;
    private readonly configPath;
    private readonly dataPath;
    constructor(client: Client, namespace: string);
    generateSpec(): Promise<PodSpec>;
    private createVolumeDirectories;
    private generatePrometheusConfig;
    private generateVolumes;
    private generateVolumesMounts;
    private generateContainersPorts;
    private generateContainers;
    private generatePodSpec;
}
