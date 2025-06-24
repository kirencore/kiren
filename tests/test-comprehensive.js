// Comprehensive Test Suite with new Test Framework
console.log("🧪 Starting Kiren Comprehensive Test Suite...\n");

// Import all test files
require('./test-url-imports.js');
require('./test-process.js');
require('./test-http.js');
require('./test-kps.js');

// Additional comprehensive tests
describe("Core JavaScript Features", () => {
    it("should support basic arithmetic", () => {
        assert.equal(2 + 2, 4);
        assert.equal(5 * 3, 15);
        assert.equal(10 / 2, 5);
    });
    
    it("should support string operations", () => {
        const str = "Hello Kiren";
        assert.ok(str.includes("Kiren"));
        assert.equal(str.length, 11);
        assert.equal(str.toUpperCase(), "HELLO KIREN");
    });
    
    it("should support arrays", () => {
        const arr = [1, 2, 3];
        assert.equal(arr.length, 3);
        assert.equal(arr[0], 1);
        arr.push(4);
        assert.equal(arr.length, 4);
    });
    
    it("should support objects", () => {
        const obj = { name: "Kiren", version: "0.2.0" };
        assert.equal(obj.name, "Kiren");
        assert.equal(obj.version, "0.2.0");
        obj.runtime = "V8";
        assert.equal(obj.runtime, "V8");
    });
    
    it("should support functions", () => {
        function add(a, b) {
            return a + b;
        }
        
        const multiply = (a, b) => a * b;
        
        assert.equal(add(2, 3), 5);
        assert.equal(multiply(4, 5), 20);
    });
});

describe("Built-in APIs", () => {
    it("should support console methods", () => {
        assert.ok(typeof console.log === "function");
        assert.ok(typeof console.error === "function");
        assert.ok(typeof console.warn === "function");
        assert.ok(typeof console.time === "function");
        assert.ok(typeof console.timeEnd === "function");
    });
    
    it("should support JSON", () => {
        const obj = { test: "value" };
        const json = JSON.stringify(obj);
        const parsed = JSON.parse(json);
        
        assert.equal(json, '{"test":"value"}');
        assert.equal(parsed.test, "value");
    });
    
    it("should support Math", () => {
        assert.equal(Math.abs(-5), 5);
        assert.equal(Math.sqrt(16), 4);
        assert.ok(Math.PI > 3.14);
        assert.ok(Math.random() >= 0 && Math.random() < 1);
    });
});

describe("Timer APIs", () => {
    it("should support setTimeout", (done) => {
        let called = false;
        setTimeout(() => {
            called = true;
        }, 10);
        
        setTimeout(() => {
            assert.ok(called, "setTimeout callback was called");
            done && done();
        }, 20);
    });
    
    it("should support setInterval", (done) => {
        let count = 0;
        const id = setInterval(() => {
            count++;
            if (count >= 2) {
                clearInterval(id);
                assert.ok(count >= 2, "setInterval called multiple times");
                done && done();
            }
        }, 10);
    });
    
    it("should support clearTimeout", () => {
        let called = false;
        const id = setTimeout(() => {
            called = true;
        }, 100);
        
        clearTimeout(id);
        
        setTimeout(() => {
            assert.equal(called, false, "Cleared timeout was not called");
        }, 150);
    });
});

describe("Module System", () => {
    it("should support CommonJS require", () => {
        const fs = require('fs');
        const path = require('path');
        const http = require('http');
        
        assert.ok(fs, "fs module loaded");
        assert.ok(path, "path module loaded");
        assert.ok(http, "http module loaded");
    });
    
    it("should have module and exports", () => {
        assert.ok(typeof module === "object", "module object exists");
        assert.ok(typeof exports === "object", "exports object exists");
        assert.ok(module.exports === exports, "module.exports === exports");
    });
});

describe("Error Handling", () => {
    it("should support try-catch", () => {
        let caught = false;
        try {
            throw new Error("Test error");
        } catch (e) {
            caught = true;
            assert.equal(e.message, "Test error");
        }
        assert.ok(caught, "Exception was caught");
    });
    
    it("should support different error types", () => {
        assert.throws(() => {
            throw new TypeError("Type error");
        });
        
        assert.throws(() => {
            throw new ReferenceError("Reference error");
        });
        
        assert.throws(() => {
            throw new SyntaxError("Syntax error");
        });
    });
});

console.log("\n🎯 Comprehensive test suite completed!");