console.log("Testing error handling...");

// Syntax error test
try {
    eval("let invalid = }"); // Syntax error
} catch (e) {
    console.log("Caught syntax error:", e.message);
}

// Runtime error test
function testFunction() {
    throw new Error("This is a test error with stack trace");
}

try {
    testFunction();
} catch (e) {
    console.log("Caught runtime error:", e.message);
    console.log("Stack trace:", e.stack);
}

// Undefined variable access
try {
    console.log(nonExistentVariable);
} catch (e) {
    console.log("Caught undefined variable error:", e.message);
}

console.log("Error handling test completed");