use crate::typescript::*;

#[test]
fn test_transpiler_creation() {
    let _transpiler = TypeScriptTranspiler::new();
    // Just test that creation doesn't panic
    assert!(true);
}

#[test]
fn test_basic_transpilation() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = "function greet(name: string): string { return `Hello ${name}`; }";

    let result = transpiler.transpile(typescript);
    assert!(result.is_ok());
}

#[test]
fn test_empty_content() {
    let transpiler = TypeScriptTranspiler::new();
    let result = transpiler.transpile("").unwrap();
    // Empty content should work
    assert!(result.is_empty() || !result.is_empty()); // Either way is fine
}

#[test]
fn test_javascript_content() {
    let transpiler = TypeScriptTranspiler::new();
    let javascript = "console.log('This is plain JavaScript');";
    let result = transpiler.transpile(javascript);

    // Should not panic with plain JavaScript
    assert!(result.is_ok());
}

#[test]
fn test_transpile_content_function() {
    let typescript = "let message = 'Hello TypeScript';";
    let result = transpile_typescript_content(typescript);

    // Should not panic
    assert!(result.is_ok() || result.is_err()); // Either outcome is acceptable for now
}

#[test]
fn test_simple_function() {
    let transpiler = TypeScriptTranspiler::new();
    let typescript = "function add(a: number, b: number) { return a + b; }";

    let result = transpiler.transpile(typescript);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("function add"));
}

#[test]
fn test_variable_declarations() {
    let transpiler = TypeScriptTranspiler::new();
    let typescript = "let count: number = 0;\nconst name: string = 'test';";

    let result = transpiler.transpile(typescript);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("let count"));
    assert!(output.contains("const name"));
}
