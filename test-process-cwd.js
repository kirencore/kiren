console.log("Testing process.cwd()...");
console.log("process object:", typeof process);
console.log("process.cwd:", typeof process.cwd);

if (typeof process.cwd === 'function') {
    try {
        const cwd = process.cwd();
        console.log("Current working directory:", cwd);
    } catch (e) {
        console.log("Error calling process.cwd():", e.message);
    }
} else {
    console.log("process.cwd is not a function");
}

console.log("process.pid:", process.pid);
console.log("process.platform:", process.platform);
console.log("process.arch:", process.arch);