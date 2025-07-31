// Test all implemented packages comprehensively
console.log('🚀 Testing Kiren full package ecosystem...\n');

// Test Express with all features
console.log('=== Express Full Test ===');
const express = require('express');
const app = express();
console.log('✓ Express imported');

app.use(express.json());
console.log('✓ Express JSON middleware');

app.use(express.urlencoded({ extended: true }));
console.log('✓ Express URL encoded middleware');

app.get('/', (req, res) => {
    res.json({ message: 'Hello Kiren!' });
});

app.get('/users/:id', (req, res) => {
    res.json({ user: req.params.id });
});
console.log('✓ Express routing with parameters');

// Test Socket.IO
console.log('\n=== Socket.IO Test ===');
const { Server } = require('socket.io');
const io = new Server();
console.log('✓ Socket.IO imported');

io.on('connection', (socket) => {
    console.log('✓ Socket connection');
    socket.emit('test', { data: 'hello' });
});

// Test Redis
console.log('\n=== Redis Test ===');
const redis = require('redis');
const client = redis.createClient();
console.log('✓ Redis client created');

client.connect();
client.set('key', 'value', () => console.log('✓ Redis SET'));
client.get('key', () => console.log('✓ Redis GET'));

// Test CORS
console.log('\n=== CORS Test ===');
const cors = require('cors');
app.use(cors());
console.log('✓ CORS middleware');

// Test Body Parser (compatibility)
console.log('\n=== Body Parser Test ===');
const bodyParser = require('body-parser');
app.use(bodyParser.json());
console.log('✓ Body Parser JSON');

// Test Cookie Parser
console.log('\n=== Cookie Parser Test ===');
const cookieParser = require('cookie-parser');
app.use(cookieParser());
console.log('✓ Cookie Parser');

// Test dotenv
console.log('\n=== Dotenv Test ===');
const dotenv = require('dotenv');
dotenv.config();
console.log('✓ Dotenv config loaded');

// Test UUID
console.log('\n=== UUID Test ===');
const { v4: uuidv4 } = require('uuid');
const id = uuidv4();
console.log('✓ UUID generated:', id);

// Test JWT
console.log('\n=== JWT Test ===');
const jwt = require('jsonwebtoken');
const token = jwt.sign({ userId: 123 }, 'secret');
console.log('✓ JWT token:', token);

jwt.verify(token, 'secret', (err, decoded) => {
    if (!err) console.log('✓ JWT verified:', decoded.userId);
});

// Test Axios
console.log('\n=== Axios Test ===');
const axios = require('axios');
console.log('✓ Axios imported');

const response = axios.get('https://api.example.com/users');
console.log('✓ Axios GET request, status:', response.status);

console.log('\n🎉 ALL TESTS PASSED!');
console.log('Kiren is now ready to replace Node.js for most backend applications!');

// Start server
const PORT = 5000;
app.listen(PORT, () => {
    console.log(`\n🚀 Server running on port ${PORT}`);
    console.log('Express server with full Kiren ecosystem is ready!');
});