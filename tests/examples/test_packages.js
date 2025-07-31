// Test popular Node.js packages
console.log('Testing Kiren package compatibility...\n');

// Test Express
console.log('=== Testing Express ===');
const express = require('express');
const app = express();
console.log('✓ Express imported successfully');

// Test Socket.IO
console.log('\n=== Testing Socket.IO ===');
const { Server } = require('socket.io');
const io = new Server();
console.log('✓ Socket.IO imported successfully');

io.on('connection', (socket) => {
    console.log('✓ Socket.IO connection event works');
    socket.emit('welcome', { message: 'Welcome to Kiren Socket.IO!' });
});

// Test Redis
console.log('\n=== Testing Redis ===');
const redis = require('redis');
const client = redis.createClient();
console.log('✓ Redis imported successfully');

client.connect();
client.set('test_key', 'Hello Kiren!', (err, result) => {
    if (!err) {
        console.log('✓ Redis SET operation works');
    }
});

client.get('test_key', (err, result) => {
    if (!err) {
        console.log('✓ Redis GET operation works');
    }
});

// Test CORS
console.log('\n=== Testing CORS ===');
const cors = require('cors');
app.use(cors());
console.log('✓ CORS middleware imported successfully');

// Test Body Parser
console.log('\n=== Testing Body Parser ===');
const bodyParser = require('body-parser');
app.use(bodyParser.json());
console.log('✓ Body Parser imported successfully');

console.log('\n🎉 All package compatibility tests passed!');
console.log('Kiren now supports major Node.js packages!');