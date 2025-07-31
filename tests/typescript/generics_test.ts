// Generic Types and Functions Test
// Tests TypeScript generics support

// Generic interface
interface Container<T> {
    value: T;
    isEmpty(): boolean;
}

// Generic class
class Box<T> implements Container<T> {
    public value: T;
    
    constructor(value: T) {
        this.value = value;
    }

    isEmpty(): boolean {
        return this.value === null || this.value === undefined;
    }

    getValue(): T {
        return this.value;
    }

    setValue(newValue: T): void {
        this.value = newValue;
    }
}

// Generic functions
function swap<T>(a: T, b: T): [T, T] {
    return [b, a];
}

function getLastItem<T>(array: T[]): T | undefined {
    return array.length > 0 ? array[array.length - 1] : undefined;
}

function combineArrays<T>(arr1: T[], arr2: T[]): T[] {
    return [...arr1, ...arr2];
}

// Multiple type parameters
function pair<T, U>(first: T, second: U): { first: T; second: U } {
    return { first, second };
}

// Constrained generics
function getProperty<T, K extends keyof T>(obj: T, key: K): T[K] {
    return obj[key];
}

// Test execution
console.log("🧬 Generics Test Starting...\n");

// Test generic class
console.log("1️⃣ Generic Class Test:");
const stringBox = new Box<string>("Hello TypeScript");
const numberBox = new Box<number>(42);
const booleanBox = new Box<boolean>(true);

console.log(`String box: ${stringBox.getValue()}, isEmpty: ${stringBox.isEmpty()}`);
console.log(`Number box: ${numberBox.getValue()}, isEmpty: ${numberBox.isEmpty()}`);
console.log(`Boolean box: ${booleanBox.getValue()}, isEmpty: ${booleanBox.isEmpty()}`);

// Test generic functions
console.log("\n2️⃣ Generic Functions Test:");
const [swapped1, swapped2] = swap("first", "second");
console.log(`Swapped strings: ${swapped1}, ${swapped2}`);

const [num1, num2] = swap(10, 20);
console.log(`Swapped numbers: ${num1}, ${num2}`);

// Test array operations
const fruits = ["apple", "banana", "orange"];
const vegetables = ["carrot", "broccoli"];
const lastFruit = getLastItem(fruits);
const combined = combineArrays(fruits, vegetables);

console.log(`Last fruit: ${lastFruit}`);
console.log(`Combined array: ${combined.join(", ")}`);

// Test multiple type parameters
console.log("\n3️⃣ Multiple Type Parameters Test:");
const stringNumberPair = pair("age", 25);
const booleanStringPair = pair(true, "active");

console.log(`String-Number pair:`, stringNumberPair);
console.log(`Boolean-String pair:`, booleanStringPair);

// Test constrained generics
console.log("\n4️⃣ Constrained Generics Test:");
const person = { name: "John", age: 30, city: "Istanbul" };
const personName = getProperty(person, "name");
const personAge = getProperty(person, "age");

console.log(`Person name: ${personName}`);
console.log(`Person age: ${personAge}`);

console.log("\n✅ Generics test completed successfully!");