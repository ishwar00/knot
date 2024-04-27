let a = "Hello, World!";

Knot.log(a, 343);
Knot.schedule_task(() => Knot.log("Hello from task"), 200);

let future = new Promise((res, _) => {
  Knot.log("executing future");
  res(455);
}).then((v) => {
  Knot.schedule_task(() => Knot.log("Hello from task then"), 200);
  Knot.log("executing future in then: ", v);
});

/// output of above code in `Knot`
/*
Hello, World! 343
executing future
executing future in then:  455
Hello from task then
Hello from task
*/
