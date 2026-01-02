// FS module test

console.log("=== FS Module Test ===\n");

// Test readFileSync
console.log("--- readFileSync ---");
const content = fs.readFileSync("examples/hello.js", "utf8");
console.log("Read hello.js, length:", content.length);
console.log("First line:", content.split("\n")[0]);

// Test existsSync
console.log("\n--- existsSync ---");
console.log("examples/hello.js exists:", fs.existsSync("examples/hello.js"));
console.log("nonexistent.txt exists:", fs.existsSync("nonexistent.txt"));

// Test writeFileSync
console.log("\n--- writeFileSync ---");
fs.writeFileSync("/tmp/kiren-test.txt", "Hello from Kiren!");
console.log("Wrote to /tmp/kiren-test.txt");
console.log("Content:", fs.readFileSync("/tmp/kiren-test.txt", "utf8"));

// Test statSync
console.log("\n--- statSync ---");
const stat = fs.statSync("examples/hello.js");
console.log("hello.js stat:", stat);
console.log("  size:", stat.size);
console.log("  isFile:", stat.isFile);
console.log("  isDirectory:", stat.isDirectory);

// Test readdirSync
console.log("\n--- readdirSync ---");
const files = fs.readdirSync("examples");
console.log("examples/ directory contents:", files);

// Test mkdirSync
console.log("\n--- mkdirSync ---");
if (!fs.existsSync("/tmp/kiren-test-dir")) {
    fs.mkdirSync("/tmp/kiren-test-dir");
    console.log("Created /tmp/kiren-test-dir");
}
console.log("Dir exists:", fs.existsSync("/tmp/kiren-test-dir"));

// Test unlinkSync and rmdirSync
console.log("\n--- unlinkSync & rmdirSync ---");
fs.unlinkSync("/tmp/kiren-test.txt");
console.log("Deleted /tmp/kiren-test.txt");
console.log("File exists after delete:", fs.existsSync("/tmp/kiren-test.txt"));

fs.rmdirSync("/tmp/kiren-test-dir");
console.log("Deleted /tmp/kiren-test-dir");
console.log("Dir exists after delete:", fs.existsSync("/tmp/kiren-test-dir"));

console.log("\n=== All FS Tests Complete ===");
