// Middleware Demo
const express = require('express');
const app = express();

console.log("Setting up Express app with middleware...");

// Global middleware
app.use((req, res, next) => {
    console.log(`${req.method} ${req.url} - ${new Date().toISOString()}`);
    req.timestamp = Date.now();
    next();
});

// JSON parsing middleware
app.use(express.json());

// Static file middleware
app.use('/public', express.static('public'));
app.use('/assets', express.static('assets', {
    maxAge: '1d',
    etag: true
}));

// Path-specific middleware
app.use('/api', (req, res, next) => {
    console.log('API middleware executed');
    req.apiCall = true;
    next();
});

// Authentication middleware for admin routes
app.use('/admin/*', (req, res, next) => {
    console.log('Admin middleware - checking authentication...');
    req.isAdmin = true;
    next();
});

// Routes with middleware
app.get('/', (req, res) => {
    res.json({
        message: 'Welcome to Kiren Express!',
        timestamp: req.timestamp,
        server: 'Kiren v0.2.0'
    });
});

app.get('/api/users', (req, res) => {
    res.json({
        users: ['Alice', 'Bob', 'Charlie'],
        apiCall: req.apiCall,
        timestamp: req.timestamp
    });
});

app.get('/admin/dashboard', (req, res) => {
    res.json({
        message: 'Admin Dashboard',
        isAdmin: req.isAdmin,
        timestamp: req.timestamp
    });
});

// Route-specific middleware
app.get('/protected', 
    (req, res, next) => {
        console.log('Authentication middleware');
        req.authenticated = true;
        next();
    },
    (req, res, next) => {
        console.log('Authorization middleware');
        req.authorized = true;
        next();
    },
    (req, res) => {
        res.json({
            message: 'Protected route accessed',
            authenticated: req.authenticated,
            authorized: req.authorized,
            timestamp: req.timestamp
        });
    }
);

// Error handling middleware
app.use((err, req, res, next) => {
    console.error('Error occurred:', err.message);
    res.status(500).json({
        error: 'Internal Server Error',
        message: err.message
    });
});

// 404 handler
app.use((req, res) => {
    res.status(404).json({
        error: 'Not Found',
        message: `Cannot ${req.method} ${req.url}`,
        timestamp: req.timestamp
    });
});

const PORT = 3000;
app.listen(PORT, () => {
    console.log(`Server running on http://localhost:${PORT}`);
    console.log('Try these endpoints:');
    console.log('  GET /');
    console.log('  GET /api/users');
    console.log('  GET /admin/dashboard');
    console.log('  GET /protected');
    console.log('  GET /public/index.html (static files)');
    console.log('  GET /nonexistent (404 test)');
});