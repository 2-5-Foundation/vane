import { DataSource } from 'typeorm';
export declare const openDb: (dbPath: string) => Promise<DataSource>;
