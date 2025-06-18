# Fetch API

The Fetch API provides a modern interface for making HTTP requests.

## Overview

Kiren's Fetch API follows the standard Web API specification and returns Promises for asynchronous operations.

## Methods

### `fetch(url, options?)`

Initiates a network request and returns a Promise.

**Parameters:**
- `url` (String) - The URL to fetch
- `options` (Object) - Optional request configuration (Coming Soon)

**Returns:**
- `Promise` - Resolves with response data

**Basic Example:**
```javascript
fetch("https://api.github.com/users/octocat")
    .then(response => {
        console.log("Request successful:", response);
    })
    .catch(error => {
        console.log("Request failed:", error);
    });
```

**With Async/Await:**
```javascript
async function fetchUser() {
    try {
        const response = await fetch("https://api.github.com/users/octocat");
        console.log("User data:", response);
    } catch (error) {
        console.log("Error:", error);
    }
}

fetchUser();
```

## Advanced Usage

### Multiple Requests
```javascript
async function fetchMultiple() {
    const urls = [
        "https://api.github.com/users/octocat",
        "https://api.github.com/users/defunkt"
    ];
    
    const promises = urls.map(url => fetch(url));
    
    try {
        const responses = await Promise.all(promises);
        console.log("All requests completed:", responses);
    } catch (error) {
        console.log("One or more requests failed:", error);
    }
}
```

### Error Handling
```javascript
fetch("https://invalid-url.example.com")
    .then(response => {
        console.log("This won't be called");
    })
    .catch(error => {
        console.log("Network error:", error);
    });
```

## Planned Features

The following features are planned for future releases:

### Request Options
```javascript
// POST request with JSON body
fetch("https://api.example.com/data", {
    method: "POST",
    headers: {
        "Content-Type": "application/json"
    },
    body: JSON.stringify({ key: "value" })
});
```

### Response Methods
```javascript
const response = await fetch("https://api.example.com/data");

// Parse JSON
const data = await response.json();

// Get text
const text = await response.text();

// Check status
if (response.ok) {
    console.log("Request successful");
}
```

## Performance Notes

- Built on the `reqwest` HTTP client library
- Requests are asynchronous and non-blocking
- Supports HTTPS out of the box
- Connection pooling for better performance

## Implementation Details

- Uses Tokio async runtime for non-blocking I/O
- Promises are implemented using V8's Promise API
- Error handling covers both network and parsing errors
- Response bodies are streamed for memory efficiency

## Browser Compatibility

Kiren's Fetch API aims to be compatible with the standard Web Fetch API:
- Similar method signatures
- Promise-based API
- Standard error handling patterns
- Compatible response object structure (when implemented)