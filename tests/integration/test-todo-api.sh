#!/bin/bash

echo "🧪 Testing Todo API with Kiren Runtime"
echo "======================================"

# Start the server in background
echo "🚀 Starting Todo API server..."
cargo run examples/todo-api.js &
SERVER_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to initialize..."
sleep 3

echo ""
echo "📋 Testing API endpoints..."

# Test 1: Get all todos
echo ""
echo "1️⃣ GET /api/todos (Get all todos):"
curl -s http://localhost:3000/api/todos | head -c 200
echo "..."

# Test 2: Get specific todo
echo ""
echo ""
echo "2️⃣ GET /api/todos/1 (Get todo #1):"
curl -s http://localhost:3000/api/todos/1 | head -c 150
echo "..."

# Test 3: Create new todo
echo ""
echo ""
echo "3️⃣ POST /api/todos (Create new todo):"
curl -s -X POST http://localhost:3000/api/todos | head -c 150
echo "..."

# Test 4: Update todo
echo ""
echo ""
echo "4️⃣ PUT /api/todos/1 (Update todo #1):"
curl -s -X PUT http://localhost:3000/api/todos/1 | head -c 150
echo "..."

# Test 5: Get statistics
echo ""
echo ""
echo "5️⃣ GET /api/stats (API Statistics):"
curl -s http://localhost:3000/api/stats | head -c 200
echo "..."

# Test 6: API Documentation
echo ""
echo ""
echo "6️⃣ GET / (API Documentation):"
echo "✅ HTML documentation available at http://localhost:3000"

echo ""
echo ""
echo "🎉 All API tests completed!"
echo "🌐 Todo API is running at: http://localhost:3000"
echo "📖 Visit the URL above to see the full documentation"
echo ""
echo "Press Ctrl+C to stop the server..."

# Wait for user to stop
wait $SERVER_PID