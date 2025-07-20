// Buffer API Test
console.log("Testing Buffer API...");

// Test Buffer.alloc
const buf1 = Buffer.alloc(10);
console.log("Buffer.alloc(10):", buf1);

// Test Buffer.allocUnsafe
const buf2 = Buffer.allocUnsafe(5);
console.log("Buffer.allocUnsafe(5):", buf2);

// Test Buffer.from with string
const buf3 = Buffer.from("hello world", "utf8");
console.log("Buffer.from('hello world'):", buf3);
console.log("toString():", buf3.toString());

// Test Buffer.from with array
const buf4 = Buffer.from([1, 2, 3, 4, 5]);
console.log("Buffer.from([1,2,3,4,5]):", buf4);

// Test Buffer.isBuffer
console.log("Buffer.isBuffer(buf1):", Buffer.isBuffer(buf1));
console.log("Buffer.isBuffer('hello'):", Buffer.isBuffer('hello'));

// Test slice
const slice = buf3.slice(0, 5);
console.log("slice(0, 5):", slice.toString());

// Test toJSON
console.log("toJSON():", JSON.stringify(buf4.toJSON()));

// Test Buffer.concat
const buf5 = Buffer.from("world");
const concatenated = Buffer.concat([buf3, buf5]);
console.log("concat result:", concatenated.toString());

// Test encodings
const base64Buf = Buffer.from("aGVsbG8=", "base64");
console.log("base64 decode:", base64Buf.toString());

const hexBuf = Buffer.from("48656c6c6f", "hex");
console.log("hex decode:", hexBuf.toString());

console.log("Buffer API tests completed!");