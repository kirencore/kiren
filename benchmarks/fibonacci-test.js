// Fibonacci hesaplama benchmark'ı
function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

console.time("fibonacci");
const result = fibonacci(35);
console.timeEnd("fibonacci");
console.log("Result:", result);