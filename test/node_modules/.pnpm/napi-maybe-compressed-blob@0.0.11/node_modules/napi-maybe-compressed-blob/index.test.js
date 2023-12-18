const maybeCompressedBlob = require('./index');
const assert = require('assert').strict;


describe("integration test", function() {
    it("should be able to compress and decompress", function() {
        const buf = Buffer.from([0x62, 0x75, 0x66, 0x66, 0x65, 0x72]); // buffer in hex
        const compressed = maybeCompressedBlob.compress(buf);
        const decompressed = maybeCompressedBlob.decompress(compressed)
        assert.equal(buf.toString(), decompressed.toString());
    });
});