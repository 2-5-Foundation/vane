import { ApiPromise } from "@polkadot/api";
import { RegisterParachainOptions } from "../types";
import { chainCustomSectionUpgrade, chainUpgradeFromLocalFile, chainUpgradeFromUrl, validateRuntimeCode } from "./chainUpgrade";
import { findPatternInSystemEventSubscription } from "./events";
import { paraGetBlockHeight, paraIsRegistered } from "./parachain";
declare function connect(apiUrl: string, types?: any): Promise<ApiPromise>;
declare function registerParachain({ id, wasmPath, statePath, apiUrl, onboardAsParachain, seed, finalization, }: RegisterParachainOptions): Promise<void>;
export { chainCustomSectionUpgrade, chainUpgradeFromLocalFile, chainUpgradeFromUrl, connect, findPatternInSystemEventSubscription, paraGetBlockHeight, paraIsRegistered, registerParachain, validateRuntimeCode, };
