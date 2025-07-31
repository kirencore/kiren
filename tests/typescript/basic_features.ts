// Basic TypeScript Features Test
// Tests fundamental TypeScript features

interface Book {
    title: string;
    author: string;
    pages: number;
    published?: Date;
}

type Genre = "fiction" | "non-fiction" | "science" | "history";

class Library {
    private books: Book[] = [];

    addBook(book: Book): void {
        this.books.push(book);
    }

    findBooksByAuthor(author: string): Book[] {
        return this.books.filter(book => book.author === author);
    }

    getTotalPages(): number {
        return this.books.reduce((total, book) => total + book.pages, 0);
    }

    getBookCount(): number {
        return this.books.length;
    }
}

function categorizeBook(book: Book, genre: Genre): string {
    return `"${book.title}" by ${book.author} is categorized as ${genre}`;
}

function createBook(title: string, author: string, pages: number): Book {
    return { title, author, pages };
}

// Test execution
console.log("📚 Basic TypeScript Features Test Starting...\n");

const library = new Library();

// Add some books
const books: Book[] = [
    createBook("1984", "George Orwell", 328),
    createBook("To Kill a Mockingbird", "Harper Lee", 281),
    { title: "The Great Gatsby", author: "F. Scott Fitzgerald", pages: 180, published: new Date("1925-04-10") }
];

books.forEach(book => library.addBook(book));

console.log(`Library has ${library.getBookCount()} books`);
console.log(`Total pages: ${library.getTotalPages()}`);

// Test genre categorization
console.log("\nBook categorization:");
console.log(categorizeBook(books[0], "fiction"));
console.log(categorizeBook(books[1], "fiction"));

// Test author search
const orwellBooks = library.findBooksByAuthor("George Orwell");
console.log(`\nBooks by George Orwell: ${orwellBooks.length}`);

console.log("\n✅ Basic features test completed!");