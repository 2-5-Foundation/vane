"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.serialize = void 0;
const baseError_1 = __importDefault(require("./baseError"));
function serialize(err) {
    const serializedObject = {
        errorClass: err.constructor.name,
        name: err.name,
        stack: err.stack,
    };
    if (err.message)
        serializedObject.message = err.message;
    if (err instanceof baseError_1.default && err.cause) {
        serializedObject.cause = serialize(err.cause);
    }
    return serializedObject;
}
exports.serialize = serialize;
