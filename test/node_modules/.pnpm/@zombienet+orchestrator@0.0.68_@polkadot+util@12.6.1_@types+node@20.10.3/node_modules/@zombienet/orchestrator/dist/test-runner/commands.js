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
const chai = require("chai");
const utils_1 = require("@zombienet/utils");
const { expect } = chai;
const Pause = ({ node_name }) => {
    return (network) => __awaiter(void 0, void 0, void 0, function* () {
        const nodes = network.getNodes(node_name);
        const results = yield Promise.all(nodes.map((node) => node.pause()));
        for (const value of results) {
            expect(value).to.be.ok;
        }
    });
};
const Resume = ({ node_name }) => {
    return (network) => __awaiter(void 0, void 0, void 0, function* () {
        const nodes = network.getNodes(node_name);
        const results = yield Promise.all(nodes.map((node) => node.resume()));
        for (const value of results) {
            expect(value).to.be.ok;
        }
    });
};
const Restart = ({ node_name, after }) => {
    after = after || 5; // at least 1 seconds
    return (network) => __awaiter(void 0, void 0, void 0, function* () {
        const nodes = network.getNodes(node_name);
        const results = yield Promise.all(nodes.map((node) => node.restart(after)));
        for (const value of results) {
            expect(value).to.be.ok;
        }
    });
};
const Sleep = ({ seconds }) => {
    seconds = seconds || 1;
    return () => __awaiter(void 0, void 0, void 0, function* () {
        yield (0, utils_1.sleep)(seconds * 1000);
        expect(true).to.be.ok;
    });
};
exports.default = {
    Pause,
    Restart,
    Resume,
    Sleep,
};
