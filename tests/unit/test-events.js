// EventEmitter API Test
console.log("Testing EventEmitter API...");

// Create EventEmitter instance
const emitter = new EventEmitter();

console.log("Max listeners default:", emitter.getMaxListeners());

// Test basic on/emit
let testResult = "";
emitter.on('test', function(data) {
    testResult = data;
    console.log("Event received:", data);
});

emitter.emit('test', 'Hello EventEmitter!');
console.log("Test result:", testResult);

// Test listener count
console.log("Listener count for 'test':", emitter.listenerCount('test'));

// Test event names
console.log("Event names:", emitter.eventNames());

// Test listeners array
console.log("Listeners for 'test':", emitter.listeners('test').length);

// Test once
let onceResult = "";
emitter.once('once-test', function(data) {
    onceResult = data;
    console.log("Once event received:", data);
});

emitter.emit('once-test', 'First emit');
console.log("Once result 1:", onceResult);

// This should not trigger the listener again
emitter.emit('once-test', 'Second emit');
console.log("Once result 2 (should be same):", onceResult);

// Test removeListener
function testListener(data) {
    console.log("Listener to be removed:", data);
}

emitter.on('remove-test', testListener);
console.log("Before remove - count:", emitter.listenerCount('remove-test'));

emitter.off('remove-test', testListener);
console.log("After remove - count:", emitter.listenerCount('remove-test'));

// Test removeAllListeners
emitter.on('clear-test', function() { console.log("Clear 1"); });
emitter.on('clear-test', function() { console.log("Clear 2"); });
console.log("Before removeAll - count:", emitter.listenerCount('clear-test'));

emitter.removeAllListeners('clear-test');
console.log("After removeAll - count:", emitter.listenerCount('clear-test'));

// Test setMaxListeners
emitter.setMaxListeners(2);
console.log("Max listeners after set:", emitter.getMaxListeners());

console.log("EventEmitter API tests completed!");