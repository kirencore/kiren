const express = require('express');
const app = express();

app.get('/healthcheck', (req, res) => {
  res.sendStatus(200);
});

app.get('/', (req, res) => {
  res.json({ 
    message: 'Hello from Kiren Docker!',
    runtime: 'Kiren v3.0.0',
    timestamp: new Date().toISOString()
  });
});

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`Kiren server running on port ${PORT}`);
});