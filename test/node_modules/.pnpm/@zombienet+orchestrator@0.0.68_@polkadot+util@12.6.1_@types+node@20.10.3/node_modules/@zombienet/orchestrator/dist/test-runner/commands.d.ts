import { Network } from "../network";
import { FnArgs } from "../types";
declare const _default: {
    Pause: ({ node_name }: FnArgs) => (network: Network) => Promise<void>;
    Restart: ({ node_name, after }: FnArgs) => (network: Network) => Promise<void>;
    Resume: ({ node_name }: FnArgs) => (network: Network) => Promise<void>;
    Sleep: ({ seconds }: FnArgs) => () => Promise<void>;
};
export default _default;
