import { Client } from "../../client";
import { Container, Volume } from "./types";
export declare class TempoResource {
    private readonly namespace;
    private readonly configPath;
    private readonly dataPath;
    constructor(client: Client, namespace: string);
    generateSpec(): Promise<{
        apiVersion: string;
        kind: string;
        metadata: {
            name: string;
            namespace: string;
            labels: {
                "zombie-role": string;
                app: string;
                "zombie-ns": string;
            };
        };
        spec: {
            hostname: string;
            restartPolicy: string;
            volumes: Volume[];
            containers: Container[];
        };
    }>;
    private createVolumeDirectories;
    private generateTempoConfig;
    private generateVolumes;
    private generateVolumesMounts;
    private generateContainersPorts;
    private generateContainers;
    private generatePodSpec;
}
