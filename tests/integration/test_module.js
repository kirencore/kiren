// Test ES Modules functionality

// Export a simple function
export function greet(name) {
    return `Hello, ${name}!`;
}

// Export a variable
export const version = "1.0.0";

// Default export
export default {
    name: "TestModule",
    greeting: greet
};