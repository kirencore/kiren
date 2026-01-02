// Heavy CPU benchmark - this should favor JIT engines
const iterations = 1000000;

function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

// Recursive fibonacci (CPU intensive)
const fib30 = fibonacci(30);

// Math heavy loop
let sum = 0;
for (let i = 0; i < iterations; i++) {
    sum += Math.sqrt(i) * Math.sin(i) * Math.cos(i);
}

// Nested loops
let matrix = 0;
for (let i = 0; i < 500; i++) {
    for (let j = 0; j < 500; j++) {
        matrix += (i * j) % 17;
    }
}

console.log("Results:", fib30, sum.toFixed(2), matrix);
