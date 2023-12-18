import { NodeMultiAddress } from "./types";
export declare function generateNodeMultiAddress(key: string, args: string[], ip: string, port: number, useWs?: boolean, certhash?: string): Promise<NodeMultiAddress>;
