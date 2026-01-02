// TextEncoder/TextDecoder Test

console.log("=== TextEncoder/TextDecoder Test ===\n");

// Test TextEncoder
console.log("--- TextEncoder ---");
const encoder = new TextEncoder();
console.log("encoding:", encoder.encoding);

const encoded = encoder.encode("Hello Kiren!");
console.log("encoded type:", encoded.constructor.name);
console.log("encoded length:", encoded.length);
console.log("encoded bytes:", Array.from(encoded).join(", "));

// Test TextDecoder
console.log("\n--- TextDecoder ---");
const decoder = new TextDecoder();
console.log("encoding:", decoder.encoding);

const decoded = decoder.decode(encoded);
console.log("decoded:", decoded);

// Test roundtrip
console.log("\n--- Roundtrip Test ---");
const original = "Merhaba Dünya! 🌍";
const enc = new TextEncoder().encode(original);
const dec = new TextDecoder().decode(enc);
console.log("original:", original);
console.log("roundtrip:", dec);
console.log("match:", original === dec);

// Test with Buffer
console.log("\n--- With Buffer ---");
const buf = Buffer.from("Buffer test");
const bufDecoded = new TextDecoder().decode(buf);
console.log("Buffer decoded:", bufDecoded);

// Test empty
console.log("\n--- Empty strings ---");
const emptyEnc = new TextEncoder().encode("");
console.log("empty encoded length:", emptyEnc.length);
const emptyDec = new TextDecoder().decode(new Uint8Array(0));
console.log("empty decoded:", "'" + emptyDec + "'");

console.log("\n=== All Encoding Tests Complete ===");
