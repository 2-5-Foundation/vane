import { PodSpec, ServiceSpec } from "./types";
export declare class ServiceResource {
    private readonly podSpec;
    constructor(podSpec: PodSpec);
    generateSpec(): ServiceSpec;
    private shouldExposeJaegerPorts;
    private generatePorts;
    private generateServiceSpec;
}
