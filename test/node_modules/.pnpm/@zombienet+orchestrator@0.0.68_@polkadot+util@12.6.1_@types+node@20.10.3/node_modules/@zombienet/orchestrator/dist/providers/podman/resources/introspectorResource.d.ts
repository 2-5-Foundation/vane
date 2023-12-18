import { PodSpec } from "./types";
export declare class IntrospectorResource {
    private readonly namespace;
    private readonly wsUri;
    constructor(namespace: string, wsUri: string);
    generateSpec(): Promise<PodSpec>;
    private generateContainersPorts;
    private generateContainers;
    private generatePodSpec;
}
