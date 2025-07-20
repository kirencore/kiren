// Test advanced dynamic import functionality

console.log("🧪 Testing Advanced Dynamic Import");
console.log("=================================");

// Test 1: File system operations via import
console.log("\n1️⃣ Testing File System via Import:");

import('fs').then(fs => {
    console.log("✅ fs module loaded");
    
    // Test writeFileSync
    try {
        fs.writeFileSync('test-import.txt', 'Hello from dynamic import!');
        console.log("✅ File written successfully");
    } catch (e) {
        console.log("❌ Write failed:", e.message);
    }
    
    // Test readFileSync
    try {
        const content = fs.readFileSync('test-import.txt');
        console.log("✅ File read successfully:", content);
    } catch (e) {
        console.log("❌ Read failed:", e.message);
    }
    
    // Test existsSync
    const exists = fs.existsSync('test-import.txt');
    console.log("✅ File exists check:", exists);
    
}).catch(err => {
    console.log("❌ fs import failed:", err.message);
});

// Test 2: Path operations
console.log("\n2️⃣ Testing Path Operations:");
import('path').then(path => {
    console.log("✅ path module loaded");
    console.log("   path.join function:", typeof path.join);
    console.log("   path.resolve function:", typeof path.resolve);
}).catch(err => {
    console.log("❌ path import failed:", err.message);
});

console.log("\n🎉 Advanced import tests initiated!");