"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getProcessStartTimeKey = exports.getMetricName = exports.getHistogramBuckets = exports.fetchMetrics = void 0;
const debug = require("debug")("zombie::metrics");
const utils_1 = require("@zombienet/utils");
const constants_1 = require("../constants");
const parseLine_1 = require("./parseLine");
// Map well know metric keys used to regex
var metricKeysMapping;
(function (metricKeysMapping) {
    metricKeysMapping["BlockHeight"] = "block_height{status=\"best\"}";
    metricKeysMapping["FinalizedHeight"] = "block_height{status=\"finalized\"}";
    metricKeysMapping["PeersCount"] = "sub_libp2p_peers_count";
})(metricKeysMapping || (metricKeysMapping = {}));
function fetchMetrics(metricUri) {
    return __awaiter(this, void 0, void 0, function* () {
        let metrics = {}; // empty by default
        try {
            debug(`fetching: ${metricUri}`);
            const fetchResult = yield fetch(metricUri, {
                signal: (0, utils_1.TimeoutAbortController)(2).signal,
                method: "GET",
                headers: {
                    accept: "application/json",
                },
            });
            if (!fetchResult.ok) {
                throw new Error(`Error - status: ${fetchResult.status}`);
            }
            const response = yield fetchResult.text();
            metrics = _extractMetrics(response);
        }
        catch (err) {
            debug(`ERR: ${err}`);
            console.log(`\n${utils_1.decorators.red(`Error`)} \t ${utils_1.decorators.bright(`fetching metrics from: ${metricUri}`)}`);
        }
        return metrics;
    });
}
exports.fetchMetrics = fetchMetrics;
function getHistogramBuckets(metricUri, metricName) {
    return __awaiter(this, void 0, void 0, function* () {
        debug(`fetching: ${metricUri}`);
        const fetchResult = yield fetch(metricUri, {
            signal: (0, utils_1.TimeoutAbortController)(2).signal,
            method: "GET",
            headers: {
                accept: "application/json",
            },
        });
        if (!fetchResult.ok) {
            throw new Error(`Error - status: ${fetchResult.status}`);
        }
        const response = yield fetchResult.text();
        let previousBucketValue = 0;
        const buckets = {};
        const resolvedMetricName = metricName.includes("_bucket")
            ? metricName
            : `${metricName}_bucket`;
        const parsedMetricInput = (0, parseLine_1.parseLine)(resolvedMetricName);
        for (const line of response.split("\n")) {
            if (line.length === 0 || line[0] === "#")
                continue; // comments and empty lines
            const parsedLine = (0, parseLine_1.parseLine)(line);
            if (parsedMetricInput.name === parsedLine.name) {
                // check labels if are presents
                let thereAreSomeMissingLabel = false;
                for (const [k, v] of parsedMetricInput.labels.entries()) {
                    console.log(`looking for key ${k}`);
                    if (!parsedLine.labels.has(k) || parsedLine.labels.get(k) !== v) {
                        thereAreSomeMissingLabel = true;
                        break;
                    }
                }
                if (thereAreSomeMissingLabel)
                    continue; // don't match
                const metricValue = parseInt(parsedLine.value);
                const leLabel = parsedLine.labels.get("le");
                buckets[leLabel] = metricValue - previousBucketValue;
                previousBucketValue = metricValue;
                debug(`${parsedLine.name} le:${leLabel} ${metricValue}`);
            }
        }
        return buckets;
    });
}
exports.getHistogramBuckets = getHistogramBuckets;
function getMetricName(metricName) {
    let metricNameTouse = metricName;
    switch (metricName) {
        case "blockheight":
        case "block height":
        case "best block":
            metricNameTouse = metricKeysMapping.BlockHeight;
            break;
        case "finalised height":
        case "finalised block":
            metricNameTouse = metricKeysMapping.FinalizedHeight;
            break;
        case "peers count":
        case "peers":
            metricNameTouse = metricKeysMapping.PeersCount;
            break;
        default:
            break;
    }
    return metricNameTouse;
}
exports.getMetricName = getMetricName;
function getProcessStartTimeKey(prefix = constants_1.DEFAULT_PROMETHEUS_PREFIX) {
    return `${prefix}_process_start_time_seconds`;
}
exports.getProcessStartTimeKey = getProcessStartTimeKey;
function _extractMetrics(text) {
    const rawMetrics = {};
    rawMetrics["_raw"] = {};
    for (const line of text.split("\n")) {
        if (line.length === 0 || line[0] === "#")
            continue; // comments and empty lines
        const parsedLine = (0, parseLine_1.parseLine)(line);
        const metricValue = parseInt(parsedLine.value);
        // get the namespace of the key
        const parts = parsedLine.name.split("_");
        const ns = parts[0];
        const rawMetricNameWithOutNs = parts.slice(1).join("_");
        const labelStrings = [];
        const labelStringsWithOutChain = [];
        for (const [k, v] of parsedLine.labels.entries()) {
            labelStrings.push(`${k}="${v}"`);
            if (k !== "chain")
                labelStringsWithOutChain.push(`${k}="${v}"`);
        }
        if (!rawMetrics[ns])
            rawMetrics[ns] = {};
        // store the metric with and without the chain
        if (labelStrings.length > 0) {
            rawMetrics[ns][`${rawMetricNameWithOutNs}{${labelStrings.join(",")}}`] =
                metricValue;
            rawMetrics["_raw"][`${parsedLine.name}{${labelStrings.join(",")}}`] =
                metricValue;
        }
        else {
            rawMetrics[ns][rawMetricNameWithOutNs] = metricValue;
            rawMetrics["_raw"][parsedLine.name] = metricValue;
        }
        if (labelStringsWithOutChain.length > 0) {
            rawMetrics[ns][`${rawMetricNameWithOutNs}{${labelStringsWithOutChain.join(",")}}`] = metricValue;
            rawMetrics["_raw"][`${parsedLine.name}{${labelStringsWithOutChain.join(",")}}`] = metricValue;
        }
        else {
            rawMetrics[ns][rawMetricNameWithOutNs] = metricValue;
            rawMetrics["_raw"][parsedLine.name] = metricValue;
        }
    }
    return rawMetrics;
}
