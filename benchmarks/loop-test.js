// Loop performance test
console.time("loop");
let sum = 0;
for (let i = 0; i < 10000000; i++) {
    sum += i;
}
console.timeEnd("loop");
console.log("Sum:", sum);