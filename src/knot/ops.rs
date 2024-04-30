use v8::{self};

use super::{task_scheduler::Task, EmbeddedData};

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
    mut retval: v8::ReturnValue,
) -> () {
    let timeout: u32 = args
        .get(1)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 0))
        .to_rust_string_lossy(scope)
        .parse()
        .unwrap_or(0);

    let context = v8::HandleScope::get_current_context(scope);
    let data = context.get_aligned_pointer_from_embedder_data(0);

    let mut global_args = vec![];

    for i in 2..args.length() {
        let global_handle = v8::Global::new(scope, args.get(i));
        global_args.push(global_handle);
    }

    let embedded_data = data as *mut EmbeddedData;
    let task = Task::Scheduled {
        callback: v8::Global::new(scope, args.get(0)),
        interval: timeout,
        args: global_args,
    };

    let task_id = unsafe {
        let mut knot_ptr = (*embedded_data).ptr.lock().unwrap();
        (**knot_ptr).scheduler.schedule(task)
    };

    retval.set_int32(task_id);
}

pub fn forget_task(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut _retval: v8::ReturnValue,
) -> () {
    let task_id: i32 = args
        .get(0)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 0))
        .to_rust_string_lossy(scope)
        .parse()
        .unwrap_or(0);

    let context = v8::HandleScope::get_current_context(scope);
    let data = context.get_aligned_pointer_from_embedder_data(0);

    let embedded_data = data as *mut EmbeddedData;

    unsafe {
        let mut knot_ptr = (*embedded_data).ptr.lock().unwrap();
        (**knot_ptr).scheduler.forget(task_id);
    };
}
