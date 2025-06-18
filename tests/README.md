# 🧪 Kiren Test Suite

This directory contains all test files for the Kiren JavaScript Runtime.

## 📁 Test Organization

### Core Test Files
- **`test-runner.js`** - Main comprehensive test suite
- **`test-suite-complete.js`** - Original complete test suite

### Feature-Specific Tests
- **`test-simple.js`** - Basic JavaScript functionality
- **`test-timers.js`** - Timer APIs (setTimeout, setInterval)
- **`test-timer-simple.js`** - Simplified timer tests
- **`test-http-simple.js`** - HTTP server functionality
- **`test-dynamic-import.js`** - ES Modules dynamic imports
- **`test-import-advanced.js`** - Advanced module import features
- **`test-new-features.js`** - Latest runtime features
- **`test-final.js`** - Final integration tests
- **`test-final-complete.js`** - Complete final test suite

### Integration Tests
- **`test-todo-api.sh`** - Shell script for Todo API testing
- **`test-dir/`** - Directory for test assets and temporary files

## 🚀 Running Tests

### Run Main Test Suite
```bash
# Core functionality tests
cargo run tests/test-runner.js

# Complete test suite  
cargo run tests/test-suite-complete.js
```

### Run Individual Tests
```bash
# Basic JavaScript tests
cargo run tests/test-simple.js

# Timer functionality
cargo run tests/test-timers.js

# HTTP server tests
cargo run tests/test-http-simple.js

# Module system tests
cargo run tests/test-dynamic-import.js
```

### Run HTTP API Tests
```bash
# Start server first
cargo run examples/todo-api-simple.js &

# Run API tests
chmod +x tests/test-todo-api.sh
./tests/test-todo-api.sh
```

## 📊 Test Coverage

The test suite covers:

### ✅ **Core JavaScript**
- Basic execution and math operations
- Functions, objects, and arrays
- String operations and manipulation
- Error handling and try-catch blocks

### ✅ **Built-in APIs**
- Console API (log, time, timeEnd)
- Timer APIs (setTimeout, setInterval, clear functions)
- File System operations via modules

### ✅ **Module Systems**
- CommonJS (require/exports)
- ES Modules (dynamic imports)
- Built-in modules (fs, path, http)

### ✅ **HTTP Server**
- Server creation and configuration
- Route registration (GET, POST, PUT, DELETE)
- Request handling and responses
- Real connection testing

### ✅ **Error Handling**
- Exception catching and reporting
- Stack trace generation
- Graceful error recovery

## 🎯 Expected Results

**Target: 100% test pass rate**

Sample output:
```
🧪 KIREN v0.1.0 - COMPREHENSIVE TEST RUNNER
=============================================

🔧 CORE JAVASCRIPT FEATURES
============================
✅ PASS: Basic JavaScript Execution
✅ PASS: Functions and Scope
✅ PASS: Objects and Arrays
✅ PASS: String Operations
✅ PASS: Math Operations

🔌 BUILT-IN APIs
================
✅ PASS: Console API
✅ PASS: Timer APIs

📦 MODULE SYSTEMS
=================
✅ PASS: CommonJS Modules
✅ PASS: Module System
✅ PASS: File System Operations

🌐 HTTP SERVER
==============
✅ PASS: HTTP Server Creation
✅ PASS: HTTP Route Registration

🚨 ERROR HANDLING
=================
✅ PASS: Error Handling
✅ PASS: Try-Catch Blocks

📊 FINAL TEST RESULTS
=====================
✅ PASSED: 12/12 tests
❌ FAILED: 0/12 tests
🎉 ALL TESTS PASSED! Kiren v0.1.0 is PRODUCTION READY!
📈 SUCCESS RATE: 100%
```

## 🔧 Debugging Failed Tests

If tests fail:

1. **Check runtime errors**: Look for stack traces in output
2. **Verify module loading**: Ensure CommonJS/ES modules work
3. **Test individual features**: Run specific test files
4. **Check HTTP server**: Verify server starts correctly
5. **Review recent changes**: Check if new code broke existing functionality

## 📝 Adding New Tests

To add new tests:

1. Create new test file in this directory
2. Follow the test function pattern:
   ```javascript
   test("Test Name", () => {
       // Test logic here
       if (condition !== expected) throw new Error("Test failed");
   });
   ```
3. Update this README with test description
4. Add to main test runner if needed

## 🎉 Success Criteria

- **All tests pass**: 100% success rate
- **No runtime errors**: Clean execution
- **Performance acceptable**: Tests complete quickly
- **Real-world usage**: HTTP API tests with curl work
- **Memory stability**: No crashes or leaks during testing