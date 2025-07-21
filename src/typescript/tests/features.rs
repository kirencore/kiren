use crate::typescript::*;

#[test]
fn test_interface_handling() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
interface User {
    name: string;
    age: number;
}

function createUser(data: User): User {
    return data;
}
"#;

    let result = transpiler.transpile(typescript);
    
    // Should not panic with interfaces
    assert!(result.is_ok());
}

#[test]
fn test_enum_handling() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
enum Color {
    Red = 1,
    Green = 2,
    Blue = 3
}
"#;

    let result = transpiler.transpile(typescript);
    
    // Should not panic with enums
    assert!(result.is_ok());
}

#[test]
fn test_class_handling() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
class Calculator {
    private value: number = 0;
    
    add(num: number): number {
        return this.value + num;
    }
}
"#;

    let result = transpiler.transpile(typescript);
    
    // Should not panic with classes
    assert!(result.is_ok());
}

#[test]
fn test_generic_handling() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
function identity<T>(arg: T): T {
    return arg;
}

class Container<T> {
    private value: T;
    
    constructor(value: T) {
        this.value = value;
    }
}
"#;

    let result = transpiler.transpile(typescript);
    
    // Should not panic with generics
    assert!(result.is_ok());
}

#[test]
fn test_arrow_function_handling() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = "const multiply = (a: number, b: number): number => a * b;";
    
    let result = transpiler.transpile(typescript);
    
    // Should not panic with arrow functions
    assert!(result.is_ok());
}

#[test]
fn test_type_alias_handling() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
type StringOrNumber = string | number;
type UserID = string;

function processId(id: UserID): StringOrNumber {
    return id;
}
"#;

    let result = transpiler.transpile(typescript);
    
    // Should not panic with type aliases
    assert!(result.is_ok());
}

#[test]
fn test_decorator_handling() {
    let transpiler = TypeScriptTranspiler::new();

    let typescript = r#"
@Component({
    selector: 'app-test'
})
class TestComponent {
    @Input() name: string = '';
    
    @Output() change = new EventEmitter();
}
"#;

    let result = transpiler.transpile(typescript);
    
    // Should not panic with decorators
    assert!(result.is_ok());
}