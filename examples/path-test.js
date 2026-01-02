// Path module test

console.log("=== Path Module Test ===\n");

// Basic properties
console.log("path.sep:", path.sep);
console.log("path.delimiter:", path.delimiter);

// basename
console.log("\n--- basename ---");
console.log("path.basename('/foo/bar/baz.txt'):", path.basename('/foo/bar/baz.txt'));
console.log("path.basename('/foo/bar/baz.txt', '.txt'):", path.basename('/foo/bar/baz.txt', '.txt'));

// dirname
console.log("\n--- dirname ---");
console.log("path.dirname('/foo/bar/baz.txt'):", path.dirname('/foo/bar/baz.txt'));
console.log("path.dirname('/foo/bar'):", path.dirname('/foo/bar'));

// extname
console.log("\n--- extname ---");
console.log("path.extname('index.html'):", path.extname('index.html'));
console.log("path.extname('index.coffee.md'):", path.extname('index.coffee.md'));
console.log("path.extname('.bashrc'):", path.extname('.bashrc'));

// join
console.log("\n--- join ---");
console.log("path.join('/foo', 'bar', 'baz'):", path.join('/foo', 'bar', 'baz'));
console.log("path.join('/foo', '../bar'):", path.join('/foo', '../bar'));

// normalize
console.log("\n--- normalize ---");
console.log("path.normalize('/foo/bar//baz/../qux'):", path.normalize('/foo/bar//baz/../qux'));

// resolve
console.log("\n--- resolve ---");
console.log("path.resolve('foo', 'bar'):", path.resolve('foo', 'bar'));
console.log("path.resolve('/foo', 'bar'):", path.resolve('/foo', 'bar'));

// isAbsolute
console.log("\n--- isAbsolute ---");
console.log("path.isAbsolute('/foo/bar'):", path.isAbsolute('/foo/bar'));
console.log("path.isAbsolute('foo/bar'):", path.isAbsolute('foo/bar'));

// parse
console.log("\n--- parse ---");
const parsed = path.parse('/home/user/file.txt');
console.log("path.parse('/home/user/file.txt'):", parsed);

// format
console.log("\n--- format ---");
console.log("path.format(parsed):", path.format(parsed));

console.log("\n=== All Tests Complete ===");
