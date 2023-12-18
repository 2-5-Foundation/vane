import { EntitySchema } from 'typeorm';
export const KeyValuePair = new EntitySchema({
    name: 'KeyValuePair',
    columns: {
        blockHash: {
            primary: true,
            type: 'varchar',
            nullable: false
        },
        key: {
            primary: true,
            type: 'varchar',
            nullable: false
        },
        value: {
            type: 'text',
            nullable: true
        }
    }
});
export const BlockEntity = new EntitySchema({
    name: 'Block',
    columns: {
        hash: {
            primary: true,
            type: 'varchar',
            nullable: false
        },
        number: {
            type: 'int',
            nullable: false
        },
        header: {
            type: 'text',
            nullable: false
        },
        parentHash: {
            type: 'varchar',
            nullable: true
        },
        extrinsics: {
            type: 'simple-array',
            nullable: false
        },
        storageDiff: {
            type: 'simple-json',
            nullable: true
        }
    }
});
