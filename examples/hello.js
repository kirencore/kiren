// Kiren Runtime - Hello World

console.log("Hello, Kiren!");
console.log("This is a JavaScript runtime.");

// Basic math
const x = 10;
const y = 20;
console.log("Sum:", x + y);

// Array and loop
const fruits = ["apple", "pear", "banana"];
console.log("Fruits:");
for (const fruit of fruits) {
    console.log("  -", fruit);
}

// Object
const person = {
    name: "John",
    age: 25,
    greet: function() {
        return `Hello, I'm ${this.name}!`;
    }
};

console.log(person.greet());

// Arrow function
const square = (n) => n * n;
console.log("Square of 5:", square(5));

// Console colors
console.info("This is an info message");
console.warn("This is a warning message");
console.error("This is an error message");

console.log("\nKiren is running successfully!");
