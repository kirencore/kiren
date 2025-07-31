// Basit TypeScript test
interface Person {
    name: string;
    age: number;
}

function greet(person: Person): string {
    return `Merhaba ${person.name}, ${person.age} yasindasin!`;
}

const kisi: Person = { name: "Ahmet", age: 25 };
console.log(greet(kisi));

// Generic fonksiyon
function getFirst<T>(items: T[]): T | undefined {
    return items[0];
}

const sayilar = [1, 2, 3, 4, 5];
const ilkSayi = getFirst(sayilar);
console.log("Ilk sayi:", ilkSayi);

const kelimeler = ["typescript", "javascript", "rust"];
const ilkKelime = getFirst(kelimeler);
console.log("Ilk kelime:", ilkKelime);

console.log("TypeScript calisiyor!");