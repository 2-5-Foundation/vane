import { z } from 'zod';
export declare const genesisSchema: z.ZodObject<{
    id: z.ZodString;
    name: z.ZodString;
    properties: z.ZodObject<{
        ss58Format: z.ZodOptional<z.ZodNumber>;
        tokenDecimals: z.ZodOptional<z.ZodUnion<[z.ZodNumber, z.ZodArray<z.ZodNumber, "many">]>>;
        tokenSymbol: z.ZodOptional<z.ZodUnion<[z.ZodString, z.ZodArray<z.ZodString, "many">]>>;
    }, "strip", z.ZodTypeAny, {
        ss58Format?: number | undefined;
        tokenDecimals?: number | number[] | undefined;
        tokenSymbol?: string | string[] | undefined;
    }, {
        ss58Format?: number | undefined;
        tokenDecimals?: number | number[] | undefined;
        tokenSymbol?: string | string[] | undefined;
    }>;
    genesis: z.ZodObject<{
        raw: z.ZodObject<{
            top: z.ZodRecord<z.ZodString, z.ZodString>;
        }, "strip", z.ZodTypeAny, {
            top: Record<string, string>;
        }, {
            top: Record<string, string>;
        }>;
    }, "strip", z.ZodTypeAny, {
        raw: {
            top: Record<string, string>;
        };
    }, {
        raw: {
            top: Record<string, string>;
        };
    }>;
}, "strip", z.ZodTypeAny, {
    name: string;
    id: string;
    properties: {
        ss58Format?: number | undefined;
        tokenDecimals?: number | number[] | undefined;
        tokenSymbol?: string | string[] | undefined;
    };
    genesis: {
        raw: {
            top: Record<string, string>;
        };
    };
}, {
    name: string;
    id: string;
    properties: {
        ss58Format?: number | undefined;
        tokenDecimals?: number | number[] | undefined;
        tokenSymbol?: string | string[] | undefined;
    };
    genesis: {
        raw: {
            top: Record<string, string>;
        };
    };
}>;
export type Genesis = z.infer<typeof genesisSchema>;
