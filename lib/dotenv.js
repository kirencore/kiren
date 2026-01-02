// Kiren dotenv Compatibility Layer
// Loads .env file into process.env

const fs = require('fs');
const path = require('path');

function config(options = {}) {
  const envPath = options.path || '.env';

  try {
    if (!fs.existsSync(envPath)) {
      return { parsed: {} };
    }

    const content = fs.readFileSync(envPath, 'utf8');
    const parsed = {};

    content.split('\n').forEach(line => {
      // Skip comments and empty lines
      line = line.trim();
      if (!line || line.startsWith('#')) return;

      // Parse KEY=VALUE
      const eqIndex = line.indexOf('=');
      if (eqIndex === -1) return;

      const key = line.slice(0, eqIndex).trim();
      let value = line.slice(eqIndex + 1).trim();

      // Remove quotes if present
      if ((value.startsWith('"') && value.endsWith('"')) ||
          (value.startsWith("'") && value.endsWith("'"))) {
        value = value.slice(1, -1);
      }

      parsed[key] = value;
      process.env[key] = value;
    });

    return { parsed };
  } catch (e) {
    return { error: e };
  }
}

module.exports = { config };
