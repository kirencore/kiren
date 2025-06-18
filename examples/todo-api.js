// Todo API - Complete HTTP Project with Kiren
// A RESTful API for managing todos

console.log("🚀 Starting Todo API Server with Kiren...");

// In-memory database (normally you'd use a real database)
let todos = [
    { id: 1, title: "Learn Kiren Runtime", completed: false, createdAt: new Date().toISOString() },
    { id: 2, title: "Build awesome projects", completed: false, createdAt: new Date().toISOString() },
    { id: 3, title: "Deploy to production", completed: false, createdAt: new Date().toISOString() }
];

let nextId = 4;

// Utility functions
function findTodoById(id) {
    return todos.find(todo => todo.id === parseInt(id));
}

function validateTodo(title) {
    return title && typeof title === 'string' && title.trim().length > 0;
}

// Create HTTP server
const server = http.createServer();

// Routes
console.log("📝 Setting up API routes...");

// GET /api/todos - Get all todos
server.get("/api/todos", () => {
    console.log("📋 GET /api/todos - Fetching all todos");
    return JSON.stringify({
        success: true,
        data: todos,
        count: todos.length,
        message: "Todos retrieved successfully"
    });
});

// GET /api/todos/{id} - Get single todo
server.get("/api/todos/1", () => {
    console.log("📄 GET /api/todos/1 - Fetching todo #1");
    const todo = findTodoById(1);
    return JSON.stringify({
        success: true,
        data: todo,
        message: "Todo retrieved successfully"
    });
});

server.get("/api/todos/2", () => {
    console.log("📄 GET /api/todos/2 - Fetching todo #2");
    const todo = findTodoById(2);
    return JSON.stringify({
        success: true,
        data: todo,
        message: "Todo retrieved successfully"
    });
});

server.get("/api/todos/3", () => {
    console.log("📄 GET /api/todos/3 - Fetching todo #3");
    const todo = findTodoById(3);
    return JSON.stringify({
        success: true,
        data: todo,
        message: "Todo retrieved successfully"
    });
});

// POST /api/todos - Create new todo
server.post("/api/todos", () => {
    console.log("➕ POST /api/todos - Creating new todo");
    
    // Simulate creating a new todo
    const newTodo = {
        id: nextId++,
        title: "New todo from API",
        completed: false,
        createdAt: new Date().toISOString()
    };
    
    todos.push(newTodo);
    
    return JSON.stringify({
        success: true,
        data: newTodo,
        message: "Todo created successfully"
    });
});

// PUT /api/todos/1 - Update todo
server.put("/api/todos/1", () => {
    console.log("✏️ PUT /api/todos/1 - Updating todo #1");
    
    const todo = findTodoById(1);
    if (todo) {
        todo.completed = !todo.completed;
        todo.updatedAt = new Date().toISOString();
        
        return JSON.stringify({
            success: true,
            data: todo,
            message: "Todo updated successfully"
        });
    } else {
        return JSON.stringify({
            success: false,
            error: "Todo not found",
            message: "Todo with id 1 not found"
        });
    }
});

// DELETE /api/todos/3 - Delete todo
server.delete("/api/todos/3", () => {
    console.log("🗑️ DELETE /api/todos/3 - Deleting todo #3");
    
    const todoIndex = todos.findIndex(todo => todo.id === 3);
    if (todoIndex !== -1) {
        const deletedTodo = todos.splice(todoIndex, 1)[0];
        
        return JSON.stringify({
            success: true,
            data: deletedTodo,
            message: "Todo deleted successfully"
        });
    } else {
        return JSON.stringify({
            success: false,
            error: "Todo not found",
            message: "Todo with id 3 not found"
        });
    }
});

// GET /api/stats - API Statistics
server.get("/api/stats", () => {
    console.log("📊 GET /api/stats - Fetching API statistics");
    
    const completedCount = todos.filter(todo => todo.completed).length;
    const pendingCount = todos.filter(todo => !todo.completed).length;
    
    return JSON.stringify({
        success: true,
        data: {
            totalTodos: todos.length,
            completedTodos: completedCount,
            pendingTodos: pendingCount,
            completionRate: todos.length > 0 ? Math.round((completedCount / todos.length) * 100) : 0,
            serverInfo: {
                runtime: "Kiren v0.1.0",
                uptime: "Active",
                apiVersion: "1.0.0"
            }
        },
        message: "Statistics retrieved successfully"
    });
});

// GET / - API Documentation
server.get("/", () => {
    console.log("📖 GET / - Serving API documentation");
    return `
    <!DOCTYPE html>
    <html>
    <head>
        <title>Todo API - Kiren Runtime</title>
        <style>
            body { font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
            .endpoint { background: #f5f5f5; padding: 15px; margin: 10px 0; border-radius: 5px; }
            .method { color: white; padding: 5px 10px; border-radius: 3px; font-weight: bold; }
            .get { background: #4CAF50; }
            .post { background: #2196F3; }
            .put { background: #FF9800; }
            .delete { background: #f44336; }
            code { background: #f0f0f0; padding: 2px 5px; border-radius: 3px; }
        </style>
    </head>
    <body>
        <h1>🚀 Todo API - Powered by Kiren Runtime</h1>
        <p>A complete RESTful API for managing todos, built with Kiren JavaScript Runtime.</p>
        
        <h2>📋 Available Endpoints:</h2>
        
        <div class="endpoint">
            <span class="method get">GET</span> <code>/api/todos</code>
            <p>Get all todos with statistics</p>
        </div>
        
        <div class="endpoint">
            <span class="method get">GET</span> <code>/api/todos/{id}</code>
            <p>Get a specific todo by ID (try /api/todos/1)</p>
        </div>
        
        <div class="endpoint">
            <span class="method post">POST</span> <code>/api/todos</code>
            <p>Create a new todo</p>
        </div>
        
        <div class="endpoint">
            <span class="method put">PUT</span> <code>/api/todos/1</code>
            <p>Toggle completion status of todo #1</p>
        </div>
        
        <div class="endpoint">
            <span class="method delete">DELETE</span> <code>/api/todos/3</code>
            <p>Delete todo #3</p>
        </div>
        
        <div class="endpoint">
            <span class="method get">GET</span> <code>/api/stats</code>
            <p>Get API statistics and server info</p>
        </div>
        
        <h2>🧪 Test the API:</h2>
        <pre>
# Get all todos
curl http://localhost:3000/api/todos

# Get specific todo
curl http://localhost:3000/api/todos/1

# Create new todo
curl -X POST http://localhost:3000/api/todos

# Update todo
curl -X PUT http://localhost:3000/api/todos/1

# Delete todo
curl -X DELETE http://localhost:3000/api/todos/3

# Get statistics
curl http://localhost:3000/api/stats
        </pre>
        
        <h2>⚡ Powered by:</h2>
        <ul>
            <li><strong>Kiren Runtime v0.1.0</strong> - High-performance JavaScript runtime</li>
            <li><strong>Built-in HTTP Server</strong> - Zero dependencies</li>
            <li><strong>Rust + V8</strong> - Memory safe and fast</li>
        </ul>
        
        <p><em>🎉 This API is running on Kiren - a production-ready JavaScript runtime built with Rust!</em></p>
    </body>
    </html>
    `;
});

// Start the server
console.log("🌐 Starting server on port 3000...");
server.listen(3000);

console.log("✅ Todo API Server is running!");
console.log("📖 Visit: http://localhost:3000");
console.log("🧪 Test API: http://localhost:3000/api/todos");
console.log("📊 Statistics: http://localhost:3000/api/stats");
console.log("");
console.log("🎯 Ready to handle requests with Kiren Runtime!");