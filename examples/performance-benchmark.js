// Performance benchmark - multiple operations
console.log("⚡ Kiren Performance Benchmark");

const start = Date.now();

// File operations
console.log("📁 Testing file operations...");
fs.writeFile("benchmark.txt", "Performance test data");
const content = fs.readFile("benchmark.txt");
const exists = fs.exists("benchmark.txt");

console.log("📄 File operations completed:", { content: content.length, exists });

// Process operations  
console.log("🔧 Testing process operations...");
const cwd = process.cwd();
const env = process.env.USER || "unknown";

console.log("🔧 Process operations completed:", { cwd: cwd.length, env });

// Timer operations
console.log("⏰ Testing timer operations...");
setTimeout(() => {
    const end = Date.now();
    const duration = end - start;
    
    console.log("🎉 Benchmark completed!");
    console.log(`⚡ Total execution time: ${duration}ms`);
    console.log(`🚀 Runtime: kiren`);
    console.log(`💡 Performance: ${duration < 100 ? 'Excellent' : duration < 500 ? 'Good' : 'Needs optimization'}`);
}, 10);

console.log("✅ Benchmark started...");