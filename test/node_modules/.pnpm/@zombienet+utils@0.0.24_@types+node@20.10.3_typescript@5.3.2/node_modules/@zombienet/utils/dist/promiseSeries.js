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
exports.series = void 0;
function series(functionsThatGeneratePromisesThatRunInSeries, concurrency = 1) {
    return __awaiter(this, void 0, void 0, function* () {
        let results = null;
        functionsThatGeneratePromisesThatRunInSeries =
            functionsThatGeneratePromisesThatRunInSeries.slice();
        return new Promise((resolve, reject) => {
            const next = (result) => {
                const concurrentPromises = [];
                results = !results ? [] : [...results, ...result];
                if (functionsThatGeneratePromisesThatRunInSeries.length) {
                    while (concurrentPromises.length < concurrency &&
                        functionsThatGeneratePromisesThatRunInSeries.length) {
                        let promise = functionsThatGeneratePromisesThatRunInSeries.shift();
                        if (typeof promise === "function") {
                            promise = promise();
                        }
                        else {
                            return reject(new Error("Invalid argument")); // see comment above. we need functions
                        }
                        if (!promise || typeof promise.then !== "function") {
                            promise = Promise.resolve(promise); // create a promise and resolve with the `promise` value.
                        }
                        concurrentPromises.push(promise);
                    }
                    Promise.all(concurrentPromises).then(next).catch(reject);
                }
                else {
                    return resolve(results);
                }
            };
            next();
        });
    });
}
exports.series = series;
