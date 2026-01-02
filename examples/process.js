// Process API test for Kiren

console.log("=== Process API Test ===\n");

// Basic info
console.log("Platform:", process.platform);
console.log("Architecture:", process.arch);
console.log("PID:", process.pid);
console.log("Version:", process.version);

// Versions object
console.log("\nVersions:");
console.log("  Kiren:", process.versions.kiren);
console.log("  QuickJS:", process.versions.quickjs);
console.log("  Zig:", process.versions.zig);

// Current working directory
console.log("\nCWD:", process.cwd());

// Command line arguments
console.log("\nArgv:");
for (let i = 0; i < process.argv.length; i++) {
    console.log(`  [${i}]:`, process.argv[i]);
}

// Environment variables
console.log("\nEnvironment (selected):");
console.log("  HOME:", process.env.HOME);
console.log("  USER:", process.env.USER);
console.log("  SHELL:", process.env.SHELL);
console.log("  PATH exists:", process.env.PATH ? "yes" : "no");

// Test process.exit (commented out)
// console.log("\nExiting with code 42...");
// process.exit(42);

console.log("\n=== Process API Test Complete ===");
