const express = require('express');
const app = express();

// Middleware
app.use(express.json());
app.use((req, res, next) => {
    console.log(`${req.method} ${req.url} - ${new Date().toISOString()}`);
    next();
});

// Routes
app.get('/', (req, res) => {
    res.json({ 
        message: 'Hello from Kiren Express!',
        runtime: 'Kiren v2.1.1',
        timestamp: new Date().toISOString()
    });
});

app.get('/api/users', (req, res) => {
    res.json({
        users: ['Alice', 'Bob', 'Charlie'],
        count: 3
    });
});

app.get('/api/users/:id', (req, res) => {
    res.json({
        user: { id: req.params.id, name: 'User ' + req.params.id },
        params: req.params
    });
});

app.post('/api/users', (req, res) => {
    res.json({
        message: 'User created',
        body: req.body
    });
});

const PORT = 3000;
app.listen(PORT, () => {
    console.log(`Server running on http://localhost:${PORT}`);
    console.log('Test endpoints:');
    console.log('  GET http://localhost:3000/');
    console.log('  GET http://localhost:3000/api/users');  
    console.log('  GET http://localhost:3000/api/users/123');
    console.log('  POST http://localhost:3000/api/users');
});