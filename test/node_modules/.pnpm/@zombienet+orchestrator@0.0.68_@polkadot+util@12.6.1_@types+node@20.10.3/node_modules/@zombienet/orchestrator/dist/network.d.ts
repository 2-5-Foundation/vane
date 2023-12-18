import { CreateLogTable } from "@zombienet/utils";
import { Metrics } from "./metrics";
import { NetworkNode } from "./networkNode";
import { Client } from "./providers/client";
export interface NodeMapping {
    [propertyName: string]: NetworkNode;
}
export interface NodeMappingMetrics {
    [propertyName: string]: Metrics;
}
export declare enum Scope {
    RELAY = 0,
    PARA = 1,
    COMPANION = 2
}
export declare function rebuildNetwork(client: Client, runningNetworkSpec: any): Network;
export declare class Network {
    relay: NetworkNode[];
    paras: {
        [id: number]: {
            chainSpecPath?: string;
            wasmPath?: string;
            statePath?: string;
            nodes: NetworkNode[];
        };
    };
    groups: {
        [id: string]: NetworkNode[];
    };
    companions: NetworkNode[];
    nodesByName: NodeMapping;
    namespace: string;
    client: Client;
    launched: boolean;
    wasRunning: boolean;
    tmpDir: string;
    backchannelUri: string;
    chainId?: string;
    chainSpecFullPath?: string;
    tracing_collator_url?: string;
    networkStartTime?: number;
    constructor(client: Client, namespace: string, tmpDir: string, startTime?: number);
    addPara(parachainId: number, chainSpecPath?: string, wasmPath?: string, statePath?: string): void;
    addNode(node: NetworkNode, scope: Scope): void;
    stop(): Promise<void>;
    dumpLogs(showLogPath?: boolean): Promise<string>;
    upsertCronJob(minutes?: number): Promise<void>;
    getBackchannelValue(key: string, timeout?: number): Promise<any>;
    getNodeByName(nodeName: string): NetworkNode;
    getNodes(nodeOrGroupName: string): NetworkNode[];
    node(nodeName: string): NetworkNode;
    nodeIsUp(nodeName: string): Promise<boolean>;
    showNetworkInfo(provider: string): void;
    showNodeInfo(node: NetworkNode, provider: string, logTable: CreateLogTable): void;
    replaceWithNetworInfo(placeholder: string): string;
    cleanMetricsCache(): void;
}
