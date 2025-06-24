// Process Management & Signals Test
console.log("Testing Kiren Process API...");

// Display process information
console.log("Process Info:");
console.log("  PID:", process.pid);
console.log("  PPID:", process.ppid);
console.log("  Platform:", process.platform);
console.log("  Architecture:", process.arch);
console.log("  Version:", process.version);
console.log("  Working Directory:", process.cwd());
console.log("  Arguments:", process.argv);

// Test memory usage
console.log("\nMemory Usage:");
const memUsage = process.memoryUsage();
console.log("  RSS:", Math.round(memUsage.rss / 1024 / 1024) + "MB");
console.log("  Heap Total:", Math.round(memUsage.heapTotal / 1024 / 1024) + "MB");
console.log("  Heap Used:", Math.round(memUsage.heapUsed / 1024 / 1024) + "MB");
console.log("  External:", Math.round(memUsage.external / 1024 / 1024) + "MB");

// Test uptime
console.log("\nUptime:", process.uptime().toFixed(2) + " seconds");

// Test hrtime
console.log("\nHigh Resolution Time:");
const start = process.hrtime();
setTimeout(() => {
    const diff = process.hrtime(start);
    console.log("  Time elapsed: " + diff[0] + "s " + diff[1] + "ns");
}, 100);

// Test process.nextTick
console.log("\nTesting process.nextTick:");
console.log("Before nextTick");
process.nextTick(() => {
    console.log("Inside nextTick callback");
});
console.log("After nextTick");

// Test signal handling
console.log("\nSetting up signal handlers...");

process.on('SIGINT', () => {
    console.log("Received SIGINT signal");
    console.log("Gracefully shutting down...");
    process.exit(0);
});

process.on('SIGTERM', () => {
    console.log("Received SIGTERM signal");
    console.log("Gracefully shutting down...");
    process.exit(0);
});

process.on('SIGUSR1', () => {
    console.log("Received SIGUSR1 signal");
    console.log("Current memory usage:", process.memoryUsage());
});

// Environment variables test
console.log("\nEnvironment Variables (first 5):");
let count = 0;
for (const [key, value] of Object.entries(process.env)) {
    if (count >= 5) break;
    console.log(`  ${key}=${value}`);
    count++;
}

console.log("\nProcess management test completed!");
console.log("Try sending signals:");
console.log("  kill -TERM " + process.pid + "  (graceful shutdown)");
console.log("  kill -INT " + process.pid + "   (interrupt)");
console.log("  kill -USR1 " + process.pid + "  (user signal)");
console.log("\nPress Ctrl+C to exit...");

// Keep process alive
setInterval(() => {
    // Just keep running
}, 1000);