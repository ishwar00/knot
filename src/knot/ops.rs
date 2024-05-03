use v8::{self};

use super::{tasks::Task, Knot};

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

pub fn schedule_periodic_task(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) -> () {
    // TODO: use serde_v8
    let interval: u32 = args
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

    let knot_ptr = data as *mut Knot;
    let callback = Task::CallBack {
        value: v8::Global::new(scope, args.get(0)),
        args: global_args,
    };
    let task_id = unsafe {
        let tasks_table = &mut (*knot_ptr).tasks_table;
        let callback_id = tasks_table.register(callback); // register the callback as task
        let id = tasks_table.register(Task::Periodic {
            interval,
            callback: callback_id,
        });
        (*knot_ptr).tasks_queue.lock().unwrap().enqueue(id);
        id
    };

    retval.set_int32(task_id);
}

// schedule_task(() => Knot.log("hey there"), 2000)
pub fn schedule_task(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) -> () {
    // TODO: use serde_v8
    let timeout: u32 = args
        .get(1)
        .to_integer(scope)
        .unwrap_or(v8::Integer::new(scope, 0))
        .to_rust_string_lossy(scope)
        .parse()
        .unwrap_or(0);

    let context = v8::HandleScope::get_current_context(scope);

    let mut global_args = vec![];

    for i in 2..args.length() {
        let global_handle = v8::Global::new(scope, args.get(i));
        global_args.push(global_handle);
    }

    let callback = Task::CallBack {
        value: v8::Global::new(scope, args.get(0)),
        args: global_args,
    };
    let data = context.get_aligned_pointer_from_embedder_data(0);
    let knot_ptr = data as *mut Knot;
    let task_id = unsafe {
        let tasks_table = &mut (*knot_ptr).tasks_table;
        let callback_id = tasks_table.register(callback); // register the callback as task
        let id = tasks_table.register(Task::Once {
            timeout,
            callback: callback_id,
        });
        (*knot_ptr).tasks_queue.lock().unwrap().enqueue(id);
        id
    };

    retval.set_int32(task_id);
}

pub fn forget_task(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut _retval: v8::ReturnValue,
) -> () {
    // TODO: use serde_v8
    if let Some(handle) = args.get(0).to_integer(scope) {
        if let Ok(task_id) = handle.to_rust_string_lossy(scope).parse::<i32>() {
            let context = v8::HandleScope::get_current_context(scope);
            let data = context.get_aligned_pointer_from_embedder_data(0);

            let knot_ptr = data as *mut Knot;
            unsafe {
                (*knot_ptr).tasks_table.unregister(&task_id);
            };
        }
    }
}
