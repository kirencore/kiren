console.log("Testing File System API...");

// Test file operations
const testFile = "test.txt";
const testContent = "Hello from Kiren File System!";

// Write file
console.log("Writing file...");
fs.writeFile(testFile, testContent);

// Check if file exists
console.log("File exists:", fs.exists(testFile));

// Read file
console.log("Reading file...");
const content = fs.readFile(testFile);
console.log("File content:", content);

// Create directory
console.log("Creating directory...");
fs.mkdir("test-dir");
console.log("Directory exists:", fs.exists("test-dir"));

console.log("File system tests completed!");