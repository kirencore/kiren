// URL-based imports test
// This demonstrates Kiren's support for URL imports like Deno

// Import from a URL
import lodash from "https://cdn.skypack.dev/lodash@4.17.21";
import axios from "https://cdn.skypack.dev/axios@1.0.0";

console.log("Testing URL imports...");

// Test lodash
const array = [1, 2, 3, 4, 5];
const doubled = lodash.map(array, n => n * 2);
console.log("Lodash map result:", doubled);

// Test axios (mock request)
const testApi = async () => {
    try {
        const response = await axios.get('https://jsonplaceholder.typicode.com/posts/1');
        console.log("API Response:", response.data.title);
    } catch (error) {
        console.log("API Error:", error.message);
    }
};

testApi();

console.log("URL import test completed!");