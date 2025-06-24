use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use v8;
use once_cell::sync::Lazy;

// Global test state
static TEST_STATE: Lazy<Arc<Mutex<TestState>>> = Lazy::new(|| {
    Arc::new(Mutex::new(TestState::new()))
});

#[derive(Debug, Clone)]
pub struct TestState {
    pub suites: Vec<TestSuite>,
    pub current_suite: Option<String>,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub skipped_tests: usize,
}

#[derive(Debug, Clone)]
pub struct TestSuite {
    pub name: String,
    pub tests: Vec<TestCase>,
    pub setup: Option<String>,
    pub teardown: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub status: TestStatus,
    pub duration: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
}

impl TestState {
    pub fn new() -> Self {
        Self {
            suites: Vec::new(),
            current_suite: None,
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            skipped_tests: 0,
        }
    }
}

/// Setup test framework in V8 context
pub fn setup_test_framework(
    scope: &mut v8::HandleScope,
    context: v8::Local<v8::Context>,
) -> Result<()> {
    let global = context.global(scope);
    
    // Add test functions
    add_test_functions(scope, &global)?;
    
    // Add assertion functions
    add_assertion_functions(scope, &global)?;
    
    // Add modern expect API
    add_expect_api(scope, &global)?;
    
    Ok(())
}

fn add_test_functions(
    scope: &mut v8::HandleScope,
    global: &v8::Local<v8::Object>,
) -> Result<()> {
    // describe(name, fn)
    let describe_key = v8::String::new(scope, "describe").unwrap();
    let describe_fn = v8::FunctionTemplate::new(scope, describe_function);
    let describe_func = describe_fn.get_function(scope).unwrap();
    global.set(scope, describe_key.into(), describe_func.into());
    
    // it(name, fn)
    let it_key = v8::String::new(scope, "it").unwrap();
    let it_fn = v8::FunctionTemplate::new(scope, it_function);
    let it_func = it_fn.get_function(scope).unwrap();
    global.set(scope, it_key.into(), it_func.into());
    
    // test(name, fn) - alias for it
    let test_key = v8::String::new(scope, "test").unwrap();
    global.set(scope, test_key.into(), it_func.into());
    
    // beforeEach(fn)
    let before_each_key = v8::String::new(scope, "beforeEach").unwrap();
    let before_each_fn = v8::FunctionTemplate::new(scope, before_each_function);
    let before_each_func = before_each_fn.get_function(scope).unwrap();
    global.set(scope, before_each_key.into(), before_each_func.into());
    
    // afterEach(fn)
    let after_each_key = v8::String::new(scope, "afterEach").unwrap();
    let after_each_fn = v8::FunctionTemplate::new(scope, after_each_function);
    let after_each_func = after_each_fn.get_function(scope).unwrap();
    global.set(scope, after_each_key.into(), after_each_func.into());
    
    Ok(())
}

fn add_assertion_functions(
    scope: &mut v8::HandleScope,
    global: &v8::Local<v8::Object>,
) -> Result<()> {
    // Create assert object
    let assert_obj = v8::Object::new(scope);
    
    // assert.equal(actual, expected)
    let equal_key = v8::String::new(scope, "equal").unwrap();
    let equal_fn = v8::FunctionTemplate::new(scope, assert_equal);
    let equal_func = equal_fn.get_function(scope).unwrap();
    assert_obj.set(scope, equal_key.into(), equal_func.into());
    
    // assert.strictEqual(actual, expected)
    let strict_equal_key = v8::String::new(scope, "strictEqual").unwrap();
    let strict_equal_fn = v8::FunctionTemplate::new(scope, assert_strict_equal);
    let strict_equal_func = strict_equal_fn.get_function(scope).unwrap();
    assert_obj.set(scope, strict_equal_key.into(), strict_equal_func.into());
    
    // assert.notEqual(actual, expected)
    let not_equal_key = v8::String::new(scope, "notEqual").unwrap();
    let not_equal_fn = v8::FunctionTemplate::new(scope, assert_not_equal);
    let not_equal_func = not_equal_fn.get_function(scope).unwrap();
    assert_obj.set(scope, not_equal_key.into(), not_equal_func.into());
    
    // assert.ok(value)
    let ok_key = v8::String::new(scope, "ok").unwrap();
    let ok_fn = v8::FunctionTemplate::new(scope, assert_ok);
    let ok_func = ok_fn.get_function(scope).unwrap();
    assert_obj.set(scope, ok_key.into(), ok_func.into());
    
    // assert.throws(fn)
    let throws_key = v8::String::new(scope, "throws").unwrap();
    let throws_fn = v8::FunctionTemplate::new(scope, assert_throws);
    let throws_func = throws_fn.get_function(scope).unwrap();
    assert_obj.set(scope, throws_key.into(), throws_func.into());
    
    // assert.doesNotThrow(fn)
    let not_throws_key = v8::String::new(scope, "doesNotThrow").unwrap();
    let not_throws_fn = v8::FunctionTemplate::new(scope, assert_does_not_throw);
    let not_throws_func = not_throws_fn.get_function(scope).unwrap();
    assert_obj.set(scope, not_throws_key.into(), not_throws_func.into());
    
    // Add assert to global
    let assert_key = v8::String::new(scope, "assert").unwrap();
    global.set(scope, assert_key.into(), assert_obj.into());
    
    Ok(())
}

fn add_expect_api(
    scope: &mut v8::HandleScope,
    global: &v8::Local<v8::Object>,
) -> Result<()> {
    // expect(value) function
    let expect_key = v8::String::new(scope, "expect").unwrap();
    let expect_fn = v8::FunctionTemplate::new(scope, expect_function);
    let expect_func = expect_fn.get_function(scope).unwrap();
    global.set(scope, expect_key.into(), expect_func.into());
    
    Ok(())
}

fn expect_function(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::String::new(scope, "expect() requires a value").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let actual_value = args.get(0);
    
    // Create an expectation object with matcher methods
    let expect_obj = v8::Object::new(scope);
    
    // Store the actual value on the object for later comparison
    let actual_key = v8::String::new(scope, "_actual").unwrap();
    expect_obj.set(scope, actual_key.into(), actual_value);
    
    // toBe matcher
    let to_be_key = v8::String::new(scope, "toBe").unwrap();
    let to_be_fn = v8::FunctionTemplate::new(scope, expect_to_be);
    let to_be_func = to_be_fn.get_function(scope).unwrap();
    expect_obj.set(scope, to_be_key.into(), to_be_func.into());
    
    // toEqual matcher (alias for toBe for now)
    let to_equal_key = v8::String::new(scope, "toEqual").unwrap();
    expect_obj.set(scope, to_equal_key.into(), to_be_func.into());
    
    rv.set(expect_obj.into());
}

fn expect_to_be(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() == 0 {
        let error = v8::String::new(scope, "toBe() requires an expected value").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let expected_value = args.get(0);
    let this_obj = args.this();
    
    // Get the actual value from the expectation object
    let actual_key = v8::String::new(scope, "_actual").unwrap();
    let actual_value = this_obj.get(scope, actual_key.into()).unwrap();
    
    // Compare values by converting to strings (simple comparison)
    let actual_str = actual_value.to_string(scope).unwrap().to_rust_string_lossy(scope);
    let expected_str = expected_value.to_string(scope).unwrap().to_rust_string_lossy(scope);
    
    if actual_str != expected_str {
        let error_msg = format!("Expected {} but got {}", expected_str, actual_str);
        let error = v8::String::new(scope, &error_msg).unwrap();
        let exception = v8::Exception::error(scope, error);
        scope.throw_exception(exception);
    }
}

// describe(name, fn)
fn describe_function(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        let error = v8::String::new(scope, "describe() requires name and function").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }
    
    let name_arg = args.get(0);
    let fn_arg = args.get(1);
    
    if !name_arg.is_string() || !fn_arg.is_function() {
        let error = v8::String::new(scope, "describe() requires string name and function").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }
    
    let name = name_arg.to_string(scope).unwrap().to_rust_string_lossy(scope);
    let test_fn = v8::Local::<v8::Function>::try_from(fn_arg).unwrap();
    
    // Set current suite
    {
        let mut state = TEST_STATE.lock().unwrap();
        state.current_suite = Some(name.clone());
        state.suites.push(TestSuite {
            name: name.clone(),
            tests: Vec::new(),
            setup: None,
            teardown: None,
        });
    }
    
    println!("\n{}", name);
    
    // Execute the test suite function
    let undefined = v8::undefined(scope);
    test_fn.call(scope, undefined.into(), &[]);
    
    // Clear current suite
    {
        let mut state = TEST_STATE.lock().unwrap();
        state.current_suite = None;
    }
}

// it(name, fn) / test(name, fn)
fn it_function(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        let error = v8::String::new(scope, "it() requires name and function").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }
    
    let name_arg = args.get(0);
    let fn_arg = args.get(1);
    
    if !name_arg.is_string() || !fn_arg.is_function() {
        let error = v8::String::new(scope, "it() requires string name and function").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }
    
    let name = name_arg.to_string(scope).unwrap().to_rust_string_lossy(scope);
    let test_fn = v8::Local::<v8::Function>::try_from(fn_arg).unwrap();
    
    // Execute the test
    let start_time = Instant::now();
    let undefined = v8::undefined(scope);
    
    let mut test_case = TestCase {
        name: name.clone(),
        status: TestStatus::Running,
        duration: None,
        error: None,
    };
    
    match test_fn.call(scope, undefined.into(), &[]) {
        Some(_) => {
            test_case.status = TestStatus::Passed;
            test_case.duration = Some(start_time.elapsed().as_millis() as u64);
            println!("    ✓ {}", name);
            
            {
                let mut state = TEST_STATE.lock().unwrap();
                state.passed_tests += 1;
                state.total_tests += 1;
            }
        }
        None => {
            test_case.status = TestStatus::Failed;
            test_case.duration = Some(start_time.elapsed().as_millis() as u64);
            
            let error_msg = "Test execution failed".to_string();
            // TODO: Implement proper exception handling for V8 0.84
            
            test_case.error = Some(error_msg.clone());
            println!("    ✗ {} - {}", name, error_msg);
            
            {
                let mut state = TEST_STATE.lock().unwrap();
                state.failed_tests += 1;
                state.total_tests += 1;
            }
        }
    }
    
    // Add test case to current suite
    {
        let mut state = TEST_STATE.lock().unwrap();
        if let Some(ref suite_name) = state.current_suite.clone() {
            if let Some(suite) = state.suites.iter_mut().find(|s| s.name == *suite_name) {
                suite.tests.push(test_case);
            }
        }
    }
}

// beforeEach(fn)
fn before_each_function(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 || !args.get(0).is_function() {
        let error = v8::String::new(scope, "beforeEach() requires a function").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }
    
    // TODO: Store setup function for current suite
}

// afterEach(fn)
fn after_each_function(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 || !args.get(0).is_function() {
        let error = v8::String::new(scope, "afterEach() requires a function").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }
    
    // TODO: Store teardown function for current suite
}

// Assertion functions

// assert.equal(actual, expected)
fn assert_equal(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        let error = v8::String::new(scope, "assert.equal() requires actual and expected values").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }
    
    let actual = args.get(0);
    let expected = args.get(1);
    
    if !actual.strict_equals(expected) {
        let actual_str = actual.to_string(scope).unwrap().to_rust_string_lossy(scope);
        let expected_str = expected.to_string(scope).unwrap().to_rust_string_lossy(scope);
        let error_msg = format!("Expected '{}', got '{}'", expected_str, actual_str);
        let error = v8::String::new(scope, &error_msg).unwrap();
        let exception = v8::Exception::error(scope, error);
        scope.throw_exception(exception);
    }
}

// assert.strictEqual(actual, expected)
fn assert_strict_equal(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    assert_equal(scope, args, _rv); // Same implementation for now
}

// assert.notEqual(actual, expected)
fn assert_not_equal(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 2 {
        let error = v8::String::new(scope, "assert.notEqual() requires actual and expected values").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }
    
    let actual = args.get(0);
    let expected = args.get(1);
    
    if actual.strict_equals(expected) {
        let actual_str = actual.to_string(scope).unwrap().to_rust_string_lossy(scope);
        let error_msg = format!("Expected values to be different, but both were '{}'", actual_str);
        let error = v8::String::new(scope, &error_msg).unwrap();
        let exception = v8::Exception::error(scope, error);
        scope.throw_exception(exception);
    }
}

// assert.ok(value)
fn assert_ok(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        let error = v8::String::new(scope, "assert.ok() requires a value").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }
    
    let value = args.get(0);
    
    if !value.is_true() && !value.boolean_value(scope) {
        let value_str = value.to_string(scope).unwrap().to_rust_string_lossy(scope);
        let error_msg = format!("Expected truthy value, got '{}'", value_str);
        let error = v8::String::new(scope, &error_msg).unwrap();
        let exception = v8::Exception::error(scope, error);
        scope.throw_exception(exception);
    }
}

// assert.throws(fn)
fn assert_throws(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 || !args.get(0).is_function() {
        let error = v8::String::new(scope, "assert.throws() requires a function").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }
    
    let test_fn = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();
    let undefined = v8::undefined(scope);
    
    let result = test_fn.call(scope, undefined.into(), &[]);
    
    if result.is_some() {
        let error = v8::String::new(scope, "Expected function to throw, but it didn't").unwrap();
        let exception = v8::Exception::error(scope, error);
        scope.throw_exception(exception);
    }
}

// assert.doesNotThrow(fn)
fn assert_does_not_throw(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 || !args.get(0).is_function() {
        let error = v8::String::new(scope, "assert.doesNotThrow() requires a function").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }
    
    let test_fn = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();
    let undefined = v8::undefined(scope);
    
    let result = test_fn.call(scope, undefined.into(), &[]);
    
    if result.is_none() {
        let error_msg = "Expected function not to throw, but it threw an error".to_string();
        let error = v8::String::new(scope, &error_msg).unwrap();
        let exception = v8::Exception::error(scope, error);
        scope.throw_exception(exception);
    }
}

/// Print test results summary
pub fn print_test_summary() {
    let state = TEST_STATE.lock().unwrap();
    
    println!("\n==========================================");
    println!("               TEST SUMMARY               ");
    println!("==========================================");
    
    for suite in &state.suites {
        println!("\n📋 {} ({} tests)", suite.name, suite.tests.len());
        for test in &suite.tests {
            let status_symbol = match test.status {
                TestStatus::Passed => "✓",
                TestStatus::Failed => "✗",
                TestStatus::Skipped => "⊝",
                _ => "?",
            };
            
            let duration_str = if let Some(duration) = test.duration {
                format!(" ({}ms)", duration)
            } else {
                String::new()
            };
            
            println!("  {} {}{}", status_symbol, test.name, duration_str);
            
            if let Some(ref error) = test.error {
                println!("    Error: {}", error);
            }
        }
    }
    
    println!("\n==========================================");
    println!("Total: {} | Passed: {} | Failed: {} | Skipped: {}", 
             state.total_tests, state.passed_tests, state.failed_tests, state.skipped_tests);
    
    if state.failed_tests > 0 {
        println!("❌ Tests failed!");
    } else {
        println!("✅ All tests passed!");
    }
    println!("==========================================\n");
}

/// Reset test state
pub fn reset_test_state() {
    let mut state = TEST_STATE.lock().unwrap();
    *state = TestState::new();
}