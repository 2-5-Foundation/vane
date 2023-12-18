import { DataSource } from 'typeorm';
import * as entities from './entities.js';
export const openDb = async (dbPath)=>{
    const source = new DataSource({
        type: 'sqlite',
        database: dbPath,
        entities: Object.values(entities),
        synchronize: true,
        logging: false,
        enableWAL: true,
        busyErrorRetry: 1000,
        busyTimeout: 5000
    });
    await source.initialize();
    return source;
};
