// Simple TypeScript Test - Her seferinde calisir

interface User {
    id: number;
    name: string;
    active?: boolean;
}

function processUser(user: User): string {
    const status = user.active !== false ? "aktif" : "pasif";
    return `${user.name} (ID: ${user.id}) - ${status}`;
}

// Generic utility function
function getFirstItem<T>(array: T[]): T | null {
    return array.length > 0 ? array[0] : null;
}

// Test data
const users: User[] = [
    { id: 1, name: "Ahmet", active: true },
    { id: 2, name: "Fatma", active: false },
    { id: 3, name: "Mehmet" } // active varsayilan olarak true
];

console.log("=== TypeScript Test Sonuclari ===");

// Test 1: Interface ve function
users.forEach(user => {
    console.log(processUser(user));
});

// Test 2: Generic function
const firstUser = getFirstItem(users);
if (firstUser) {
    console.log(`\nIlk kullanici: ${firstUser.name}`);
}

const numbers = [10, 20, 30, 40, 50];
const firstNumber = getFirstItem(numbers);
console.log(`Ilk sayi: ${firstNumber}`);

// Test 3: Union types ve optional properties
type Theme = "dark" | "light";
const currentTheme: Theme = "dark";
console.log(`\nMevcut tema: ${currentTheme}`);

// Test 4: Array methods with types
const activeUsers = users.filter(user => user.active !== false);
const userNames = activeUsers.map(user => user.name);
console.log(`Aktif kullanicilar: ${userNames.join(", ")}`);

console.log("\n✅ TypeScript testi basariyla tamamlandi!");