import { openDB } from 'idb';
export class IdbDatabase {
    datasource;
    constructor(location){
        this.datasource = openDB(location, 1, {
            upgrade (db) {
                db.createObjectStore('keyValue');
                const blockStore = db.createObjectStore('block', {
                    keyPath: 'hash'
                });
                blockStore.createIndex('byNumber', 'number');
            }
        });
    }
    async close() {
        const db = await this.datasource;
        db.close();
    }
    async saveBlock(block) {
        const db = await this.datasource;
        const tx = db.transaction([
            'block'
        ], 'readwrite');
        const store = tx.objectStore('block');
        store.delete(block.hash);
        store.put(block);
        await tx.done;
    }
    async queryBlock(hash) {
        const db = await this.datasource;
        const block = await db.get('block', hash);
        return block ?? null;
    }
    async queryBlockByNumber(number) {
        const db = await this.datasource;
        const block = await db.getFromIndex('block', 'byNumber', number);
        return block ?? null;
    }
    async queryHighestBlock() {
        const db = await this.datasource;
        const index = db.transaction('block').store.index('byNumber');
        const cursor = await index.openCursor(null, 'prev');
        return cursor?.value ?? null;
    }
    async deleteBlock(hash) {
        const db = await this.datasource;
        await db.delete('block', hash);
    }
    async blocksCount() {
        const db = await this.datasource;
        return db.count('block');
    }
    async saveStorage(blockHash, key, value) {
        const db = await this.datasource;
        await db.put('keyValue', value, `${blockHash}-${key}`);
    }
    async queryStorage(blockHash, key) {
        const db = await this.datasource;
        const value = await db.get('keyValue', `${blockHash}-${key}`);
        return value !== undefined ? {
            blockHash,
            key,
            value
        } : null;
    }
}
