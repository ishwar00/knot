use core::panic;
use std::{
    ffi::c_void,
    sync::{Arc, Mutex, Once},
};
use v8::{self};

use self::task_scheduler::TaskScheduler;
mod ops;
mod task_scheduler;

const KNOT_INIT: Once = Once::new();

pub struct Knot<'a, 'b> {
    context: v8::Local<'a, v8::Context>,
    context_scope: v8::ContextScope<'b, v8::HandleScope<'a>>,
    scheduler: TaskScheduler,
}
type V8Instance = v8::OwnedIsolate;

struct EmbeddedData<'a, 'b> {
    ptr: Arc<Mutex<*mut Knot<'a, 'b>>>,
}

impl<'a, 'b> Knot<'a, 'b>
where
    'a: 'b,
{
    pub fn init_v8<'i>() -> V8Instance {
        // TODO: I don't know what make_shared does
        KNOT_INIT.call_once(|| {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
        });

        let isolate = v8::Isolate::new(v8::CreateParams::default());
        isolate
    }

    pub fn new(handle_scope: &'b mut v8::HandleScope<'a, ()>) -> Box<Self> {
        let global_template = Knot::create_glob_template(handle_scope);

        let knot_template = v8::ObjectTemplate::new(handle_scope);
        knot_template.set(
            v8::String::new(handle_scope, "Knot").unwrap().into(),
            global_template.into(),
        );

        let context = v8::Context::new_from_template(handle_scope, knot_template);
        let context_scope = v8::ContextScope::new(handle_scope, context);
        let task_scheduler = TaskScheduler::new();
        let mut self_ = Box::new(Self {
            context_scope,
            context,
            scheduler: task_scheduler,
        });

        let knot_ptr = self_.as_mut() as *mut Self;

        let embedded_data = Box::new(EmbeddedData {
            ptr: Arc::new(std::sync::Mutex::new(knot_ptr)),
        });

        let embedded_data_ptr = Box::leak(embedded_data);

        unsafe {
            context.set_aligned_pointer_in_embedder_data(
                0,
                embedded_data_ptr as *mut EmbeddedData as *mut c_void,
            )
        };

        self_
    }

    pub fn run_tasks(&mut self) -> () {
        let global = self.context.global(&mut self.context_scope);
        while let Some(task_id) = self.scheduler.fetch_expired_task() {
            if let Some(task) = self.scheduler.fetch_task(task_id) {
                match task {
                    task_scheduler::Task::Scheduled { callback, args, .. } => {
                        let value = v8::Local::new(&mut self.context_scope, callback);
                        let callback_fn = v8::Local::<v8::Function>::try_from(value)
                            .expect("Task callback must be a function!");

                        let mut args_buff = vec![];
                        for arg in args {
                            let local_handle = v8::Local::new(&mut self.context_scope, arg);
                            args_buff.push(local_handle);
                        }

                        callback_fn.call(&mut self.context_scope, global.into(), &args_buff);
                    }
                }
            }
        }
    }

    pub fn run_microtasks(&mut self) -> () {
        self.context_scope.perform_microtask_checkpoint();
    }

    pub fn run_event_loop(&mut self) -> () {
        loop {
            self.run_microtasks();
            self.run_tasks();
            if !self.scheduler.has_pending_tasks() {
                break;
            }
        }
    }

    pub fn execute_script(&mut self, script: String) {
        let script = v8::String::new(&mut self.context_scope, &script).unwrap();
        let scope = &mut v8::HandleScope::new(&mut self.context_scope);
        let try_catch = &mut v8::TryCatch::new(scope);

        let script =
            v8::Script::compile(try_catch, script, None).expect("Failed to run the script.");

        if script.run(try_catch).is_none() {
            let exception = try_catch.exception().unwrap();
            let exception_str = exception
                .to_string(try_catch)
                .unwrap()
                .to_rust_string_lossy(try_catch);

            panic!("{}", exception_str);
        }
    }

    fn create_glob_template<'i, 'c>(
        scope: &'c mut v8::HandleScope<'i, ()>,
    ) -> v8::Local<'i, v8::ObjectTemplate> {
        let global = v8::ObjectTemplate::new(scope);

        global.set(
            v8::String::new(scope, "log").unwrap().into(),
            v8::FunctionTemplate::new(scope, ops::print).into(),
        );

        global.set(
            v8::String::new(scope, "schedule_task").unwrap().into(),
            v8::FunctionTemplate::new(scope, ops::schedule_task).into(),
        );

        global.set(
            v8::String::new(scope, "forget_task").unwrap().into(),
            v8::FunctionTemplate::new(scope, ops::forget_task).into(),
        );

        global
    }
}
