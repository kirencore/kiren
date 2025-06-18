console.log("Hello from Kiren!");
console.log("2 + 2 =", 2 + 2);

const name = "World";
console.log(`Hello, ${name}!`);

function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

console.log("Fibonacci(10) =", fibonacci(10));