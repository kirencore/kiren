console.log("Testing Fetch API...");

// Test fetch API
fetch("https://api.github.com/users/mertcanaltin")
    .then(response => console.log("Fetch completed:", response))
    .catch(error => console.log("Fetch error:", error));

console.log("Fetch request initiated");