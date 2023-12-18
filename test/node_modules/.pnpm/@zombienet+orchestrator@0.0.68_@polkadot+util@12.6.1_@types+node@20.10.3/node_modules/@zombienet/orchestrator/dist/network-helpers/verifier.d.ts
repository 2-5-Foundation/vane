import { Network } from "../network";
import { NetworkNode } from "../networkNode";
export declare const nodeChecker: (node: NetworkNode) => Promise<number | undefined>;
export declare function verifyNodes(network: Network): Promise<void>;
