let a = "Hello, World!";

Knot.log(a, 343);
let taskId = Knot.scheduleTask(() => {
    Knot.log("Hello from task");
    new Promise((r, _) => {
        Knot.log("Nested bruh");
        r(45);
    });
}, 5000);

let future = new Promise((res, _) => {
    Knot.log("executing future");
    res(455);
}).then((v) => {
    let _ = Knot.scheduleTask(() => Knot.log("Hello from task then"), 2000);
    Knot.log("executing future in then: ", v);
});

Knot.log(future);
Knot.forgetTask(taskId);
// Knot.schedulePeriodicTask(() => Knot.log("periodic"), 1000);

// Hello, World! 343
// executing future
// [object Promise]
// executing future in then:  455
// Hello from task then
// Hello from task
// Nested bruh
