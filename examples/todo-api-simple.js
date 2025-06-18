// Simple Todo API - Kiren Runtime Example
console.log("🚀 Starting Simple Todo API with Kiren...");

// Create HTTP server
const server = http.createServer();

// Routes
console.log("📝 Setting up API routes...");

// GET /api/todos - Get all todos
server.get("/api/todos", "GET /api/todos: [{'id':1,'title':'Learn Kiren','done':false},{'id':2,'title':'Build API','done':true}]");

// GET /api/todos/1 - Get specific todo
server.get("/api/todos/1", "GET /api/todos/1: {'id':1,'title':'Learn Kiren Runtime','completed':false,'created':'2024-01-01'}");

// POST /api/todos - Create new todo
server.post("/api/todos", "POST /api/todos: {'id':3,'title':'New Todo','completed':false,'message':'Todo created successfully'}");

// PUT /api/todos/1 - Update todo
server.put("/api/todos/1", "PUT /api/todos/1: {'id':1,'title':'Learn Kiren Runtime','completed':true,'message':'Todo updated successfully'}");

// DELETE /api/todos/2 - Delete todo
server.delete("/api/todos/2", "DELETE /api/todos/2: {'message':'Todo deleted successfully','deleted_id':2}");

// GET /api/stats - Statistics
server.get("/api/stats", "GET /api/stats: {'total_todos':3,'completed':1,'pending':2,'api_version':'1.0','runtime':'Kiren v0.1.0'}");

// GET / - Documentation
server.get("/", `
<!DOCTYPE html>
<html>
<head>
    <title>Todo API - Kiren Runtime</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; background: #f5f5f5; }
        .container { background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; }
        .endpoint { background: #ecf0f1; padding: 15px; margin: 15px 0; border-radius: 8px; border-left: 4px solid #3498db; }
        .method { color: white; padding: 8px 15px; border-radius: 5px; font-weight: bold; display: inline-block; margin-right: 10px; }
        .get { background: #27ae60; }
        .post { background: #3498db; }
        .put { background: #f39c12; }
        .delete { background: #e74c3c; }
        code { background: #34495e; color: white; padding: 5px 10px; border-radius: 5px; font-family: 'Courier New', monospace; }
        .highlight { background: #fff3cd; padding: 15px; border-radius: 5px; border-left: 4px solid #ffc107; margin: 20px 0; }
        .footer { text-align: center; margin-top: 30px; color: #7f8c8d; font-style: italic; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Todo API - Powered by Kiren Runtime</h1>
        <p>A simple RESTful API for managing todos, demonstrating the power of <strong>Kiren JavaScript Runtime</strong>.</p>
        
        <div class="highlight">
            <strong>🎯 What is Kiren?</strong><br>
            Kiren is a high-performance JavaScript runtime built with Rust and V8 engine. 
            It provides a lightweight, memory-safe alternative for server-side JavaScript execution.
        </div>
        
        <h2>📋 Available API Endpoints:</h2>
        
        <div class="endpoint">
            <span class="method get">GET</span> <code>/api/todos</code>
            <p><strong>Get all todos</strong> - Returns a list of all todos in the system</p>
        </div>
        
        <div class="endpoint">
            <span class="method get">GET</span> <code>/api/todos/1</code>
            <p><strong>Get specific todo</strong> - Returns details for todo with ID 1</p>
        </div>
        
        <div class="endpoint">
            <span class="method post">POST</span> <code>/api/todos</code>
            <p><strong>Create new todo</strong> - Creates a new todo item</p>
        </div>
        
        <div class="endpoint">
            <span class="method put">PUT</span> <code>/api/todos/1</code>
            <p><strong>Update todo</strong> - Updates the completion status of todo #1</p>
        </div>
        
        <div class="endpoint">
            <span class="method delete">DELETE</span> <code>/api/todos/2</code>
            <p><strong>Delete todo</strong> - Removes todo #2 from the system</p>
        </div>
        
        <div class="endpoint">
            <span class="method get">GET</span> <code>/api/stats</code>
            <p><strong>API Statistics</strong> - Returns API usage statistics and server info</p>
        </div>
        
        <h2>🧪 Test the API:</h2>
        <pre style="background: #2c3e50; color: #ecf0f1; padding: 20px; border-radius: 8px; overflow-x: auto;">
# Get all todos
curl http://localhost:3000/api/todos

# Get specific todo
curl http://localhost:3000/api/todos/1

# Create new todo  
curl -X POST http://localhost:3000/api/todos

# Update todo
curl -X PUT http://localhost:3000/api/todos/1

# Delete todo
curl -X DELETE http://localhost:3000/api/todos/2

# Get statistics
curl http://localhost:3000/api/stats
        </pre>
        
        <h2>⚡ Technology Stack:</h2>
        <ul style="line-height: 1.8;">
            <li><strong>Runtime:</strong> Kiren v0.1.0 (Rust + V8)</li>
            <li><strong>HTTP Server:</strong> Built-in Hyper-based server</li>
            <li><strong>Performance:</strong> Memory-safe, zero-dependency</li>
            <li><strong>Deployment:</strong> Single binary, easy to distribute</li>
        </ul>
        
        <div class="highlight">
            <strong>🎉 Why Kiren?</strong><br>
            • <strong>Fast startup:</strong> Minimal overhead compared to Node.js<br>
            • <strong>Memory safe:</strong> Rust prevents memory leaks and crashes<br>
            • <strong>Single binary:</strong> No complex dependency management<br>
            • <strong>Production ready:</strong> Built for real-world applications
        </div>
        
        <div class="footer">
            <p>🚀 This API is running on <strong>Kiren Runtime</strong> - The future of server-side JavaScript!</p>
            <p>Built with ❤️ using Rust, V8, and modern web standards.</p>
        </div>
    </div>
</body>
</html>
`);

// Start the server
console.log("🌐 Starting server on port 3000...");
server.listen(3000);

console.log("✅ Todo API Server is running!");
console.log("📖 Visit: http://localhost:3000");
console.log("🧪 Test API: http://localhost:3000/api/todos");
console.log("📊 Statistics: http://localhost:3000/api/stats");
console.log("");
console.log("🎯 Ready to handle requests with Kiren Runtime!");