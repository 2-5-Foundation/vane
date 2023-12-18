import { ApiPromise } from "@polkadot/api";
export declare function chainUpgradeFromUrl(api: ApiPromise, wasmFileUrl: string): Promise<string>;
export declare function chainUpgradeFromLocalFile(api: ApiPromise, filePath: string): Promise<string>;
export declare function chainCustomSectionUpgrade(api: ApiPromise): Promise<string>;
export declare function validateRuntimeCode(api: ApiPromise, paraId: number, hash: string, timeout?: number): Promise<boolean>;
