use core::time;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    thread::sleep,
};

pub type JsValueRef = v8::Global<v8::Value>;

pub struct TaskScheduler {
    tasks: HashMap<i32, Task>,
    expired: Arc<Mutex<VecDeque<i32>>>,
    next_task_id: i32,
}

impl TaskScheduler {
    pub fn new() -> Self {
        let mutexed_queue = std::sync::Mutex::new(VecDeque::new());
        Self {
            tasks: HashMap::new(),
            next_task_id: 0,
            expired: Arc::new(mutexed_queue),
        }
    }

    pub fn fetch_task(&mut self, task_id: i32) -> Option<Task> {
        self.tasks.remove(&task_id)
    }

    fn task_id(&mut self) -> i32 {
        self.next_task_id += 1;
        self.next_task_id
    }

    pub fn has_pending_tasks(&self) -> bool {
        let expired = self.expired.lock().unwrap().len() > 0;
        let schedule_tasks = self.tasks.len() > 0;
        expired || schedule_tasks
    }

    pub fn fetch_expired_task(&mut self) -> Option<i32> {
        let mut expired = self.expired.lock().unwrap();
        expired.pop_front()
    }

    pub fn forget(&mut self, task_id: i32) -> () {
        // we are not removing from expired container
        // will simply ignore if we could not find task in HashMap
        self.tasks.remove(&task_id);
    }

    pub fn schedule(&mut self, task: Task) -> i32 {
        let task_id = self.task_id();
        let Task::Scheduled { interval, .. } = &task;
        let interval = (*interval).into();
        let expired = self.expired.clone();

        std::thread::spawn(move || {
            sleep(time::Duration::from_millis(interval));
            let mut expired = expired.lock().unwrap();
            expired.push_back(task_id);
        });
        self.tasks.insert(task_id, task);
        task_id
    }
}

pub enum Task {
    Scheduled {
        callback: JsValueRef,
        interval: u32,
        args: Vec<JsValueRef>,
    },
}
