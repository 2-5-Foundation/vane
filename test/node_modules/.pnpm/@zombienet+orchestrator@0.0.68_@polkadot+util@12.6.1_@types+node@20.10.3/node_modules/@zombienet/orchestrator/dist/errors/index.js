"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const baseError_1 = __importDefault(require("./baseError"));
const serializer_1 = require("./serializer");
class orchestratorError extends baseError_1.default {
}
const errors = {
    orchestratorError,
};
exports.default = {
    serialize: serializer_1.serialize,
    errors,
};
