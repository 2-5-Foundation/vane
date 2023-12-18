export default class BaseError extends Error {
    causedByMessage: string;
    cause?: Error | undefined;
    constructor(...args: any);
    fullStack(): string | undefined;
    _parseArguments(args: any): {
        cause: Error | undefined;
        message: string;
    };
}
