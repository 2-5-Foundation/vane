export declare function sleep(ms: number): Promise<unknown>;
export declare function retry(delayMs: number, timeoutMs: number, fn: () => Promise<boolean | undefined>, errMsg: string): Promise<void>;
export declare function generateNamespace(n?: number): string;
export declare function getSha256(input: string): string;
export declare function addMinutes(howMany: number, baseDate?: Date): [number, number];
export declare const convertBytes: (bytes: number) => string;
export declare function isValidHttpUrl(input: string): boolean;
export declare function filterConsole(excludePatterns: string[], options?: any): () => void;
export declare function convertExponentials(data: string): string;
export declare function getLokiUrl(namespace: string, podName: string, from: number | string, to?: number | string): string;
export declare const TimeoutAbortController: (time: number) => AbortController;
export declare function getRandom(arr: string[], n: number): any[];
export declare function getFilePathNameExt(filePath: string): {
    fullPath: string;
    fileName: string;
    extension: string;
};
export declare function validateImageUrl(image: string): string;
