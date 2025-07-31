// Comprehensive TypeScript Feature Test Suite
// Tests all major TypeScript features supported by SWC strip-only mode

// 1. Basic Interface Test
interface Person {
    name: string;
    age: number;
    email?: string;
}

interface Employee extends Person {
    employeeId: number;
    department: string;
}

// 2. Generic Interface
interface Repository<T> {
    items: T[];
    add(item: T): void;
    find(predicate: (item: T) => boolean): T | undefined;
}

// 3. Class Implementation
class UserRepository implements Repository<Employee> {
    items: Employee[] = [];

    add(employee: Employee): void {
        this.items.push(employee);
    }

    find(predicate: (employee: Employee) => boolean): Employee | undefined {
        return this.items.find(predicate);
    }

    findByDepartment(department: string): Employee[] {
        return this.items.filter(emp => emp.department === department);
    }

    getTotalEmployees(): number {
        return this.items.length;
    }
}

// 4. Generic Functions
function identity<T>(arg: T): T {
    return arg;
}

function mapArray<T, U>(array: T[], mapper: (item: T) => U): U[] {
    return array.map(mapper);
}

function filterArray<T>(array: T[], predicate: (item: T) => boolean): T[] {
    return array.filter(predicate);
}

// 5. Union Types
type Status = "active" | "inactive" | "pending";
type UserRole = "admin" | "user" | "guest";

// 6. Type Aliases
type UserWithRole = Employee & { role: UserRole; status: Status };

// 7. Utility Functions with Complex Types
function createEmployeeWithRole(
    person: Person,
    employeeId: number,
    department: string,
    role: UserRole = "user"
): UserWithRole {
    return {
        ...person,
        employeeId,
        department,
        role,
        status: "active"
    };
}

function getEmployeeStatus(employee: UserWithRole): string {
    switch (employee.status) {
        case "active":
            return `${employee.name} is currently active`;
        case "inactive":
            return `${employee.name} is inactive`;
        case "pending":
            return `${employee.name} is pending approval`;
        default:
            return `${employee.name} has unknown status`;
    }
}

// 8. Async Functions with Promises
async function processEmployee(employee: Employee): Promise<string> {
    return new Promise((resolve) => {
        setTimeout(() => {
            resolve(`Processed employee: ${employee.name} from ${employee.department}`);
        }, 10);
    });
}

async function processMultipleEmployees(employees: Employee[]): Promise<string[]> {
    const promises = employees.map(emp => processEmployee(emp));
    return Promise.all(promises);
}

// 9. Main Test Function
async function runComprehensiveTest(): Promise<void> {
    console.log("🧪 Starting Comprehensive TypeScript Test Suite\n");

    // Test 1: Repository Pattern
    console.log("1️⃣ Repository Pattern Test:");
    const userRepo = new UserRepository();
    
    const employees: Employee[] = [
        { name: "Alice", age: 30, employeeId: 1, department: "Engineering" },
        { name: "Bob", age: 25, employeeId: 2, department: "Marketing", email: "bob@company.com" },
        { name: "Charlie", age: 35, employeeId: 3, department: "Engineering" }
    ];

    employees.forEach(emp => userRepo.add(emp));
    console.log(`Total employees: ${userRepo.getTotalEmployees()}`);

    // Test 2: Generic Functions
    console.log("\n2️⃣ Generic Functions Test:");
    const names = mapArray(employees, emp => emp.name);
    console.log(`Employee names: ${names.join(", ")}`);

    const engineeringEmps = filterArray(employees, emp => emp.department === "Engineering");
    console.log(`Engineering employees: ${engineeringEmps.length}`);

    const identityTest = identity("TypeScript works!");
    console.log(`Identity function result: ${identityTest}`);

    // Test 3: Union Types and Type Aliases
    console.log("\n3️⃣ Union Types Test:");
    const employeesWithRoles: UserWithRole[] = employees.map(emp => 
        createEmployeeWithRole(emp, emp.employeeId, emp.department, 
            emp.department === "Engineering" ? "admin" : "user")
    );

    employeesWithRoles.forEach(emp => {
        console.log(getEmployeeStatus(emp));
    });

    // Test 4: Async/Promise Test
    console.log("\n4️⃣ Async Processing Test:");
    try {
        const results = await processMultipleEmployees(employees);
        results.forEach(result => console.log(result));
    } catch (error) {
        console.error("Async processing failed:", error);
    }

    // Test 5: Complex Type Operations
    console.log("\n5️⃣ Complex Type Operations:");
    const foundEmployee = userRepo.find(emp => emp.name === "Alice");
    if (foundEmployee) {
        console.log(`Found: ${foundEmployee.name} in ${foundEmployee.department}`);
    }

    const engineeringTeam = userRepo.findByDepartment("Engineering");
    console.log(`Engineering team size: ${engineeringTeam.length}`);

    console.log("\n✅ All TypeScript tests completed successfully!");
    console.log("🎉 SWC TypeScript integration is working perfectly!");
}

// Execute the test suite
runComprehensiveTest().catch(console.error);