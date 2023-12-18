import { ILoader } from "nunjucks";
export declare class RelativeLoader implements ILoader {
    private paths;
    constructor(paths: string[]);
    getSource(name: string): {
        src: string;
        path: string;
        noCache: boolean;
    };
}
