// Buffer API Test

console.log("=== Buffer API Test ===\n");

// Test Buffer.alloc
console.log("--- Buffer.alloc ---");
const buf1 = Buffer.alloc(10);
console.log("Buffer.alloc(10) length:", buf1.length);
console.log("buf1.at(0) (should be 0):", buf1.at(0));

// Test Buffer.from with string
console.log("\n--- Buffer.from(string) ---");
const buf2 = Buffer.from("Hello Kiren!");
console.log("Buffer.from('Hello Kiren!') length:", buf2.length);
console.log("toString():", buf2.toString());

// Test Buffer.from with array
console.log("\n--- Buffer.from(array) ---");
const buf3 = Buffer.from([72, 105, 33]); // "Hi!"
console.log("Buffer.from([72, 105, 33]):", buf3.toString());

// Test slice
console.log("\n--- slice ---");
const buf4 = Buffer.from("Hello World");
const sliced = buf4.slice(0, 5);
console.log("'Hello World'.slice(0, 5):", sliced.toString());

// Test copy (simplified)
console.log("\n--- copy ---");
const source = Buffer.from("ABCD");
const target = Buffer.alloc(6);
const copied = source.copy(target, 1);
console.log("copied bytes:", copied);
console.log("target:", target.toString());

// Test write
console.log("\n--- write ---");
const buf5 = Buffer.alloc(12);
buf5.write("Hello");
buf5.write("World", 6);
console.log("write result:", buf5.toString());

// Test readUInt8/writeUInt8
console.log("\n--- readUInt8/writeUInt8 ---");
const buf6 = Buffer.alloc(4);
buf6.writeUInt8(0xDE, 0);
buf6.writeUInt8(0xAD, 1);
buf6.writeUInt8(0xBE, 2);
buf6.writeUInt8(0xEF, 3);
console.log("bytes:", buf6.readUInt8(0), buf6.readUInt8(1), buf6.readUInt8(2), buf6.readUInt8(3));

// Test equals
console.log("\n--- equals ---");
const bufA = Buffer.from("test");
const bufB = Buffer.from("test");
const bufC = Buffer.from("Test");
console.log("'test' equals 'test':", bufA.equals(bufB));
console.log("'test' equals 'Test':", bufA.equals(bufC));

// Test fill
console.log("\n--- fill ---");
const buf7 = Buffer.alloc(10);
buf7.fill(65); // 'A'
console.log("fill(65):", buf7.toString());

// Test Buffer.concat
console.log("\n--- Buffer.concat ---");
const part1 = Buffer.from("Hello ");
const part2 = Buffer.from("World");
const part3 = Buffer.from("!");
const combined = Buffer.concat([part1, part2, part3]);
console.log("concat:", combined.toString());

// Test Buffer.isBuffer
console.log("\n--- Buffer.isBuffer ---");
console.log("isBuffer(Buffer.alloc(1)):", Buffer.isBuffer(Buffer.alloc(1)));
console.log("isBuffer('string'):", Buffer.isBuffer("string"));
console.log("isBuffer({}):", Buffer.isBuffer({}));

// Test Buffer.byteLength
console.log("\n--- Buffer.byteLength ---");
console.log("byteLength('Hello'):", Buffer.byteLength("Hello"));
console.log("byteLength(Buffer.alloc(10)):", Buffer.byteLength(Buffer.alloc(10)));

console.log("\n=== All Buffer Tests Complete ===");
