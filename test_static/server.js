const express = require('express');
const app = express();

// Serve static files from 'public' directory
app.use(express.static('public'));

// API route
app.get('/api/hello', function(req, res) {
  res.json({ message: 'Hello from API!' });
});

app.listen(3000, function() {
  console.log('Server running on http://localhost:3000');
  console.log('Try:');
  console.log('  curl http://localhost:3000/');
  console.log('  curl http://localhost:3000/style.css');
  console.log('  curl http://localhost:3000/api/hello');
});
