// Modern ES Modules ile Kiren File System örneği
console.log("📁 Starting Modular File Operations Demo...");

// FS modülünü import et
import("kiren/fs").then(async (fs) => {
    console.log("✅ FS module loaded successfully");
    
    // Test dosyası yolu
    const testFile = "modular-test.txt";
    const testDir = "modular-test-dir";
    
    console.log("🔧 Starting file operations...");
    
    try {
        // 1. Dosya yazma
        console.log("✏️  Writing file...");
        fs.writeFile(testFile, "Hello from Modular Kiren FS! 🎯\nThis file was created using ES Modules.");
        console.log(`✅ File '${testFile}' written successfully`);
        
        // 2. Dosya okuma
        console.log("📖 Reading file...");
        const content = fs.readFile(testFile);
        console.log("📄 File content:");
        console.log(content);
        
        // 3. Dosya varlığını kontrol etme
        console.log("🔍 Checking file existence...");
        const exists = fs.exists(testFile);
        console.log(`📝 File '${testFile}' exists: ${exists}`);
        
        // 4. Dizin oluşturma
        console.log("📁 Creating directory...");
        fs.mkdir(testDir);
        console.log(`✅ Directory '${testDir}' created successfully`);
        
        // 5. Dizin varlığını kontrol etme
        const dirExists = fs.exists(testDir);
        console.log(`📂 Directory '${testDir}' exists: ${dirExists}`);
        
        // 6. Dizin içine dosya yazma
        const nestedFile = `${testDir}/nested-file.txt`;
        console.log("📝 Writing nested file...");
        fs.writeFile(nestedFile, "This is a nested file created with modular FS API!");
        
        const nestedContent = fs.readFile(nestedFile);
        console.log("📄 Nested file content:");
        console.log(nestedContent);
        
        console.log("🎉 All modular file operations completed successfully!");
        
    } catch (error) {
        console.error("❌ File operation failed:", error);
    }
    
}).catch(error => {
    console.error("❌ Failed to load FS module:", error);
});