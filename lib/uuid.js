// Kiren UUID Compatibility Layer
// Uses native crypto.randomUUID()

function v4() {
  return crypto.randomUUID();
}

module.exports = { v4 };
