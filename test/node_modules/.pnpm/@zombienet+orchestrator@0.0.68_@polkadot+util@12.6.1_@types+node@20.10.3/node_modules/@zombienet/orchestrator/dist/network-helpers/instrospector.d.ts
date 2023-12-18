import { NetworkNode } from "../networkNode";
import { Client } from "../providers/client";
export declare function spawnIntrospector(client: Client, node: NetworkNode, inCI?: boolean): Promise<NetworkNode>;
