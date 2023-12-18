import { Client } from "../../client";
import { PodSpec } from "./types";
export declare class GrafanaResource {
    private readonly namespace;
    private readonly prometheusIp;
    private readonly tempoIp;
    private readonly dataSourcesPath;
    constructor(client: Client, namespace: string, prometheusIp: string, tempoIp: string);
    generateSpec(): Promise<PodSpec>;
    private createVolumeDirectories;
    private generateGrafanaConfig;
    private generateVolumes;
    private generateVolumesMounts;
    private generateContainersPorts;
    private generateContainers;
    private generatePodSpec;
}
