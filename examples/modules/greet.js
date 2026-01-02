// Greeting module

function greet(name) {
  return `Hello, ${name}!`;
}

function farewell(name) {
  return `Goodbye, ${name}!`;
}

// Export using module.exports
module.exports = {
  greet,
  farewell,
  version: "1.0.0"
};
