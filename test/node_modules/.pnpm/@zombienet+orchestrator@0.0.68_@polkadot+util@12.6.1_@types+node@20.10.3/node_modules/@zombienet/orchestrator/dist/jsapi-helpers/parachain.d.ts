import { ApiPromise } from "@polkadot/api";
export declare function paraGetBlockHeight(api: ApiPromise, paraId: number): Promise<number>;
export declare function paraIsRegistered(api: ApiPromise, paraId: number): Promise<boolean>;
