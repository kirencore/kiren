console.log("Starting simple import test");

// Test direct async function call
(async () => {
    console.log("About to call import()");
    const result = await import('kiren/http');
    console.log("Import result:", result);
    console.log("createServer function:", result.createServer);
})().catch(err => {
    console.error("Import error:", err);
});

console.log("Simple import test setup complete");