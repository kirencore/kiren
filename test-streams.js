// Streams API Test
console.log("Testing Streams API...");

// Test if stream module exists
console.log("Stream module available:", typeof stream);

if (typeof stream !== 'undefined') {
    console.log("Stream constructors available:");
    console.log("- Readable:", typeof stream.Readable);
    console.log("- Writable:", typeof stream.Writable);
    console.log("- Transform:", typeof stream.Transform);
    console.log("- PassThrough:", typeof stream.PassThrough);

    // Test Readable stream
    try {
        const readable = new stream.Readable();
        console.log("Readable stream created:", !!readable);
        console.log("Readable has read method:", typeof readable.read === 'function');
        console.log("Readable has push method:", typeof readable.push === 'function');
        console.log("Readable has pause method:", typeof readable.pause === 'function');
        console.log("Readable has resume method:", typeof readable.resume === 'function');
        console.log("Readable has pipe method:", typeof readable.pipe === 'function');
    } catch (e) {
        console.log("Readable stream error:", e.message);
    }

    // Test Writable stream
    try {
        const writable = new stream.Writable();
        console.log("Writable stream created:", !!writable);
        console.log("Writable has write method:", typeof writable.write === 'function');
        console.log("Writable has end method:", typeof writable.end === 'function');
        console.log("Writable has cork method:", typeof writable.cork === 'function');
        console.log("Writable has uncork method:", typeof writable.uncork === 'function');
    } catch (e) {
        console.log("Writable stream error:", e.message);
    }

    // Test Transform stream
    try {
        const transform = new stream.Transform();
        console.log("Transform stream created:", !!transform);
        console.log("Transform has _transform method:", typeof transform._transform === 'function');
        console.log("Transform has _flush method:", typeof transform._flush === 'function');
    } catch (e) {
        console.log("Transform stream error:", e.message);
    }

    // Test PassThrough stream
    try {
        const passthrough = new stream.PassThrough();
        console.log("PassThrough stream created:", !!passthrough);
        console.log("PassThrough has _transform method:", typeof passthrough._transform === 'function');
    } catch (e) {
        console.log("PassThrough stream error:", e.message);
    }

    // Test basic stream operations
    try {
        const readable = new stream.Readable();
        
        // Test push/read
        readable.push("Hello ");
        readable.push("World!");
        readable.push(null); // End stream
        
        const chunk1 = readable.read();
        console.log("Read chunk 1:", chunk1);
        
        const chunk2 = readable.read();
        console.log("Read chunk 2:", chunk2);
        
        console.log("Stream read test completed");
    } catch (e) {
        console.log("Stream operations error:", e.message);
    }
} else {
    console.log("Stream module not available");
}

console.log("Streams API tests completed!");