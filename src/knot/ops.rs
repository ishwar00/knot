use std::ffi::c_void;

use v8::{self, Handle};

pub fn print(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut _retval: v8::ReturnValue,
) {
    for i in 0..args.length() {
        let mut arg_i = args
            .get(i)
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope);

        if i + 1 < args.length() {
            arg_i.push(' ');
        }

        print!("{}", arg_i)
    }
    println!();
}

// schedule_task(() => Knot.log("hey there"), 2000)
pub fn schedule_task(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut _retval: v8::ReturnValue,
) -> () {
    let timeout: i32 = args
        .get(1)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 0))
        .to_rust_string_lossy(scope)
        .parse()
        .unwrap_or(0);

    let context = v8::HandleScope::get_current_context(scope);
    let data = context.get_aligned_pointer_from_embedder_data(0);

    let task_queue = data as *mut Vec<crate::knot::Task>;
    let task = v8::Global::new(scope, args.get(0));
    unsafe {
        (*task_queue).push(task);
    }
}
