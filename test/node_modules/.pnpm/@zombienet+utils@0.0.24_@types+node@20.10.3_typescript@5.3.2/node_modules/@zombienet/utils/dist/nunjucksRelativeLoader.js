"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.RelativeLoader = void 0;
const fs_1 = require("fs");
class RelativeLoader {
    constructor(paths) {
        this.paths = paths;
    }
    getSource(name) {
        const fullPath = require.resolve(name, {
            paths: this.paths,
        });
        return {
            src: (0, fs_1.readFileSync)(fullPath, "utf-8"),
            path: fullPath,
            noCache: true,
        };
    }
}
exports.RelativeLoader = RelativeLoader;
