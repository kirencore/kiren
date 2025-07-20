// Test importing from ES module

import { greet, version } from './test_module.js';
import testModule from './test_module.js';

console.log("Testing ES Module imports...");

console.log("Named imports:");
console.log("greet('World'):", greet('World'));
console.log("version:", version);

console.log("Default import:");
console.log("testModule:", testModule);
console.log("testModule.name:", testModule.name);

console.log("ES Module test completed!");