const http = require('http');

const server = http.createServer((req, res) => {
  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({
    message: 'Kiren Docker test server',
    path: req.url,
    timestamp: new Date().toISOString()
  }));
});

const PORT = 3000;
server.listen(PORT, () => {
  console.log(`Kiren test server running on port ${PORT}`);
});