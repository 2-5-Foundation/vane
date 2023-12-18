import BaseError from "./baseError";
import { serialize } from "./serializer";
declare class orchestratorError extends BaseError {
}
declare const _default: {
    serialize: typeof serialize;
    errors: {
        orchestratorError: typeof orchestratorError;
    };
};
export default _default;
