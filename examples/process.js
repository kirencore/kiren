console.log("Testing Process API...");

// Test process.argv
console.log("Command line arguments:");
console.log("process.argv:", process.argv);

// Test process.env
console.log("\nEnvironment variables:");
console.log("HOME:", process.env.HOME);
console.log("PATH exists:", typeof process.env.PATH !== "undefined");

// Test process.cwd
console.log("\nCurrent working directory:");
console.log("process.cwd():", process.cwd());

console.log("\nProcess API tests completed!");