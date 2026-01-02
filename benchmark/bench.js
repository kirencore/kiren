// Simple benchmark script
const iterations = 100000;

// Math operations
let sum = 0;
for (let i = 0; i < iterations; i++) {
    sum += Math.sqrt(i) * Math.sin(i);
}

// String operations
let str = "";
for (let i = 0; i < 1000; i++) {
    str += "hello";
}

// Array operations
const arr = [];
for (let i = 0; i < 10000; i++) {
    arr.push(i * 2);
}
const filtered = arr.filter(x => x % 4 === 0);

// Object operations
const obj = {};
for (let i = 0; i < 1000; i++) {
    obj["key" + i] = i * i;
}

console.log("Done:", sum.toFixed(2), str.length, filtered.length, Object.keys(obj).length);
