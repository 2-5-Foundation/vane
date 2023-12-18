export interface Metrics {
    [propertyName: string]: {
        [propertyName: string]: number;
    };
}
export interface BucketHash {
    [le: string]: number;
}
export declare function fetchMetrics(metricUri: string): Promise<Metrics>;
export declare function getHistogramBuckets(metricUri: string, metricName: string): Promise<BucketHash>;
export declare function getMetricName(metricName: string): string;
export declare function getProcessStartTimeKey(prefix?: string): string;
