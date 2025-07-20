# 🧪 Kiren Test Suite

This directory contains all test files for the Kiren JavaScript Runtime.

## 📁 Test Organization

### Unit Tests (`unit/`)
Core functionality and individual feature tests:
- **Buffer API Tests**: `test-buffer.js`
- **Events API Tests**: `test-events.js` 
- **Process API Tests**: `test-process.js`, `test-process-cwd.js`
- **Streams API Tests**: `test-streams.js`
- **Timer API Tests**: `test-timers.js`, `test_timer.js`, `test_timer_full.js`
- **Error Handling Tests**: `test_error.js`
- **Comprehensive Test Suites**: `test-comprehensive.js`, `test-final-complete.js`, `test-final.js`, `test-new-features.js`, `test-suite-complete.js`

### Integration Tests (`integration/`)
End-to-end and system integration tests:
- **HTTP Server Tests**: `test-http-simple.js`, `test-http.js`, `test_http_server.js`
- **Static File Tests**: `test-static-files.js`
- **Module System Tests**: `test-import-advanced.js`, `test-module-resolution.js`, `test-dynamic-import.js`, `test-url-imports.js`, `test_import.js`, `test_module.js`
- **Compatibility Tests**: `test-gurubu-compatibility.js`
- **Middleware Tests**: `test-middleware.js`
- **API Tests**: `test-kps.js`, `test-runner.js`, `test-todo-api.sh`

### Performance Tests (`performance/`)
Benchmarking and performance validation:
- **Core Performance**: `perf_test.js`
- **Timer Performance**: `test-timer-simple.js`

### Examples (`examples/`)
Simple usage examples and basic tests:
- **Basic Tests**: `simple-test.js`, `simple_http_test.js`, `test-simple.js`
- **Test Assets**: `test-file.txt`, `test-dir/`

## 🚀 Running Tests

### Run Main Test Suite
```bash
# Core functionality tests
cargo run tests/integration/test-runner.js

# Complete test suite  
cargo run tests/unit/test-suite-complete.js
```

### Run Unit Tests
```bash
# Timer functionality
cargo run tests/unit/test-timers.js

# Buffer API tests
cargo run tests/unit/test-buffer.js

# Events API tests
cargo run tests/unit/test-events.js

# Process API tests
cargo run tests/unit/test-process.js

# Error handling tests
cargo run tests/unit/test_error.js
```

### Run Integration Tests
```bash
# HTTP server tests
cargo run tests/integration/test-http-simple.js

# Module system tests
cargo run tests/integration/test-dynamic-import.js

# Compatibility tests
cargo run tests/integration/test-gurubu-compatibility.js
```

### Run Performance Tests
```bash
# Core performance benchmarks
cargo run tests/performance/perf_test.js

# Timer performance
cargo run tests/performance/test-timer-simple.js
```

### Run Example Tests
```bash
# Basic JavaScript tests
cargo run tests/examples/test-simple.js

# Simple HTTP test
cargo run tests/examples/simple_http_test.js
```

### Run HTTP API Tests
```bash
# Start server first
cargo run examples/todo-api-simple.js &

# Run API tests
chmod +x tests/integration/test-todo-api.sh
./tests/integration/test-todo-api.sh
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