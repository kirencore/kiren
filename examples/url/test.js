// URL API Test

console.log("=== URL API Test ===\n");

// Test URL parsing
console.log("--- URL parsing ---");
const url1 = new URL("https://example.com:8080/path/to/page?foo=bar&baz=qux#section");
console.log("href:", url1.href);
console.log("protocol:", url1.protocol);
console.log("hostname:", url1.hostname);
console.log("port:", url1.port);
console.log("pathname:", url1.pathname);
console.log("search:", url1.search);
console.log("hash:", url1.hash);
console.log("origin:", url1.origin);

// Test simple URL
console.log("\n--- Simple URL ---");
const url2 = new URL("http://localhost/api/users");
console.log("href:", url2.href);
console.log("hostname:", url2.hostname);
console.log("pathname:", url2.pathname);

// Test URL toString
console.log("\n--- URL toString ---");
console.log("url1.toString():", url1.toString());

// Test URLSearchParams
console.log("\n--- URLSearchParams ---");
const params1 = URLSearchParams("foo=1&bar=2&foo=3");
console.log("get('foo'):", params1.get("foo"));
console.log("getAll('foo'):", params1.getAll("foo"));
console.log("has('bar'):", params1.has("bar"));
console.log("has('baz'):", params1.has("baz"));

// Test URLSearchParams set
console.log("\n--- URLSearchParams set ---");
const params2 = URLSearchParams();
params2.set("name", "Kiren");
params2.set("version", "0.1.0");
console.log("toString():", params2.toString());

// Test URLSearchParams append
console.log("\n--- URLSearchParams append ---");
params2.append("feature", "fast");
params2.append("feature", "small");
console.log("after append:", params2.toString());
console.log("getAll('feature'):", params2.getAll("feature"));

// Test URLSearchParams delete
console.log("\n--- URLSearchParams delete ---");
params2.delete("feature");
console.log("after delete:", params2.toString());

// Test URL searchParams
console.log("\n--- URL searchParams ---");
const url3 = new URL("https://api.example.com/search?q=kiren&limit=10&sort=name");
console.log("searchParams.get('q'):", url3.searchParams.get("q"));
console.log("searchParams.get('limit'):", url3.searchParams.get("limit"));
console.log("searchParams.has('sort'):", url3.searchParams.has("sort"));

// Test with leading ?
console.log("\n--- URLSearchParams with leading ? ---");
const params3 = URLSearchParams("?key=value");
console.log("get('key'):", params3.get("key"));

console.log("\n=== All URL Tests Complete ===");
