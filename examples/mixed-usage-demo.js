// Hem modüler hem global API kullanımı örneği
console.log("🔄 Starting Mixed Usage Demo (Modular + Global)...");

// Global API'leri kullan (backward compatibility)
console.log("🌐 Using global APIs:");
console.log("  - process.env.USER:", process.env.USER || "unknown");
console.log("  - process.cwd():", process.cwd());

// Global fetch kullan
console.log("🌍 Testing global fetch API...");
fetch("https://api.github.com/users/github")
  .then(response => {
    console.log("✅ Global fetch working - Status:", response.status);
  })
  .catch(error => {
    console.log("⚠️  Global fetch failed:", error.message);
  });

// Global timer kullan
setTimeout(() => {
  console.log("⏰ Global setTimeout working!");
}, 1000);

// ES Modules ile HTTP ve FS import et
Promise.all([
  import("kiren/http"),
  import("kiren/fs")
]).then(([http, fs]) => {
  console.log("🎯 Modular imports loaded successfully!");
  
  // Modüler FS ile dosya oluştur
  const configFile = "mixed-config.json";
  const config = {
    serverPort: 3002,
    environment: "development",
    features: {
      globalAPIs: true,
      modularImports: true,
      mixedUsage: true
    },
    createdAt: new Date().toISOString()
  };
  
  console.log("💾 Creating config file with modular FS...");
  fs.writeFile(configFile, JSON.stringify(config, null, 2));
  
  // Modüler HTTP ile server oluştur
  console.log("🚀 Creating HTTP server with modular import...");
  const server = http.createServer();
  
  // Routes
  server.get("/", () => {
    return "Mixed Usage Demo - Global + Modular APIs working together! 🤝";
  });
  
  server.get("/config", () => {
    // Modüler FS ile config dosyasını oku
    const configContent = fs.readFile(configFile);
    return JSON.parse(configContent);
  });
  
  server.get("/system", () => {
    // Global process API kullan
    return {
      runtime: "kiren",
      usage: "mixed (global + modular)",
      environment: process.env.NODE_ENV || "development",
      cwd: process.cwd(),
      args: process.argv
    };
  });
  
  // Server'ı başlat
  server.listen(config.serverPort);
  
  console.log("🎉 Mixed usage demo ready!");
  console.log(`📋 Server running on port ${config.serverPort}`);
  console.log("🔗 Available endpoints:");
  console.log(`  - GET  http://localhost:${config.serverPort}/`);
  console.log(`  - GET  http://localhost:${config.serverPort}/config`);
  console.log(`  - GET  http://localhost:${config.serverPort}/system`);
  console.log(`  - GET  http://localhost:${config.serverPort}/health`);
  
}).catch(error => {
  console.error("❌ Failed to load modular APIs:", error);
});

console.log("✨ Mixed usage demo initialized - combining the best of both worlds!");