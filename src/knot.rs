use core::panic;
use std::{
    ffi::c_void,
    sync::{Arc, Mutex, Once},
    thread::sleep,
    time,
};
use v8::{self};

use self::tasks::{Task, TaskId, TasksQueue, TasksTable};

mod ops;
pub(crate) mod tasks;

const KNOT_INIT: Once = Once::new();

pub struct Knot<'a, 'b> {
    context: v8::Local<'a, v8::Context>,
    context_scope: v8::ContextScope<'b, v8::HandleScope<'a>>,
    tasks_table: Arc<Mutex<TasksTable>>,
    tasks_queue: Arc<Mutex<TasksQueue>>,
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
        let mut self_ = Box::new(Self {
            context_scope,
            context,
            tasks_table: Arc::new(Mutex::new(TasksTable::new())),
            tasks_queue: Arc::new(Mutex::new(TasksQueue::new())),
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

    pub fn register(&mut self, task: Task) -> TaskId {
        self.tasks_table.lock().unwrap().register(task)
    }

    pub fn enqueue(&mut self, item: TaskId) -> () {
        self.tasks_queue.lock().unwrap().0.push_back(item);
    }

    fn run_microtasks(&mut self) -> () {
        self.context_scope.perform_microtask_checkpoint();
    }

    fn has_tasks(&self) -> bool {
        self.tasks_queue.lock().unwrap().0.len() > 0
    }

    pub fn run_event_loop(&mut self) -> () {
        while self.has_tasks() {
            // Just to make sure that we don't hold onto the lock longer than required
            {
                let task_id = {
                    let mut tasks_queue = self.tasks_queue.lock().unwrap();
                    let task_id = tasks_queue.0.pop_front().unwrap();
                    tasks_queue.dequeue(); // remove the task from the queue
                    task_id
                };
                let mut tasks_table = self.tasks_table.lock().unwrap();
                let task = tasks_table
                    .as_mut(&task_id)
                    .expect("Knot internal error: queue has taskId but table does not have task");

                match task {
                    Task::Once { timeout, callback } => {
                        let tasks_queue = self.tasks_queue.clone();
                        let timeout: u64 = (*timeout).into();
                        let callback_id: i32 = (*callback).into();
                        std::thread::spawn(move || {
                            sleep(time::Duration::from_millis(timeout));
                            tasks_queue.lock().unwrap().enqueue(callback_id);
                        });
                    }
                    Task::Periodic { interval, callback } => {
                        let tasks_queue = self.tasks_queue.clone();
                        let timeout: u64 = (*interval).into();
                        let callback_id: i32 = (*callback).into();
                        std::thread::spawn(move || {
                            sleep(time::Duration::from_millis(timeout));
                            let mut tasks_queue = tasks_queue.lock().unwrap();
                            tasks_queue.enqueue(task_id);
                            tasks_queue.enqueue(callback_id); // scheduling again
                        });
                    }
                    Task::Script { source } => {
                        Self::execute_script(&mut self.context_scope, source);
                    }
                    Task::CallBack { value, args } => {
                        let value = value.clone();
                        let global = self.context.global(&mut self.context_scope);
                        let value = v8::Local::new(&mut self.context_scope, value.clone());
                        let callback_fn = v8::Local::<v8::Function>::try_from(value)
                            .expect("Task callback must be a function!");

                        let mut args_buff = vec![];
                        for arg in args {
                            let local_handle = v8::Local::new(&mut self.context_scope, arg.clone());
                            args_buff.push(local_handle);
                        }

                        callback_fn.call(&mut self.context_scope, global.into(), &args_buff);
                    }
                }
            }
            self.run_microtasks();
        }
    }

    fn execute_script(context_scope: &mut v8::ContextScope<'b, v8::HandleScope<'a>>, script: &str) {
        let script = v8::String::new(context_scope, &script).unwrap();
        let scope = &mut v8::HandleScope::new(context_scope);
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
            v8::String::new(scope, "scheduleTask").unwrap().into(),
            v8::FunctionTemplate::new(scope, ops::schedule_task).into(),
        );

        global.set(
            v8::String::new(scope, "schedulePeriodicTask")
                .unwrap()
                .into(),
            v8::FunctionTemplate::new(scope, ops::schedule_periodic_task).into(),
        );

        global.set(
            v8::String::new(scope, "forgetTask").unwrap().into(),
            v8::FunctionTemplate::new(scope, ops::forget_task).into(),
        );

        global
    }
}
