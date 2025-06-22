// Test advanced logging and debugging features
console.log("Testing Kiren v2.0 Logging System");

console.info("This is an info message");
console.warn("This is a warning");
console.error("This is an error");
console.debug("This is a debug message");

console.trace("Trace test");

console.assert(true, "This assertion should pass");
console.assert(false, "This assertion should fail");

// Test object logging
const testObj = { name: "test", value: 42 };
console.log("Object:", testObj);

// Test timer
console.time("test-timer");
console.timeEnd("test-timer");

console.clear();
console.log("Console cleared - this should be visible");