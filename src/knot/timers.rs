use core::time;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    thread::sleep,
};

pub struct Timers {
    scheduler: Scheduler,
    tasks_table: Table,
}

impl Timers {
    pub fn new() -> Self {
        Self {
            scheduler: Scheduler::new(),
            tasks_table: Table::new(),
        }
    }

    pub fn schedule(&mut self, task: Task, strategy: SchedulingTrategy) -> i32 {
        let task_id = self.tasks_table.add(task);
        self.scheduler.schedule(task_id, strategy);
        task_id
    }

    pub fn has_pending_timers(&self) -> bool {
        // No particular reason to OR here, I just don't like || asthetically
        let left_timers = self.scheduler.timers.lock().unwrap().len();
        let expired_timers = self.scheduler.expired.lock().unwrap().len();

        (left_timers | expired_timers) > 0
    }

    pub fn remove(&mut self, task_id: i32) -> () {
        self.tasks_table.remove(task_id);
        self.scheduler.forget(task_id);
    }

    pub fn fetch_expired_timer(&mut self) -> Option<Task> {
        if let Some(task_id) = self.scheduler.fetch_expired_timer() {
            let strat = self
                .scheduler
                .timers
                .lock()
                .unwrap()
                .get(&task_id)
                .unwrap()
                .clone();
            match strat {
                SchedulingTrategy::Once(_) => {
                    let task = self.tasks_table.table.remove(&task_id);
                    self.remove(task_id);
                    task
                }
                SchedulingTrategy::Periodic(_) => {
                    Some(self.tasks_table.table.get(&task_id).unwrap().clone())
                }
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub callback: v8::Global<v8::Value>,
    pub args: Vec<v8::Global<v8::Value>>,
}

type Interval = u32;

#[derive(Clone)]
pub enum SchedulingTrategy {
    Once(Interval),
    Periodic(Interval),
}

pub type TimerId = i32;

pub struct Table {
    table: HashMap<TimerId, Task>,
    next_task_id: i32,
}

impl Table {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            next_task_id: 0,
        }
    }

    fn gen_id(&mut self) -> i32 {
        self.next_task_id += 1;
        self.next_task_id
    }

    pub fn add(&mut self, callback: Task) -> i32 {
        let task_id = self.gen_id();
        self.table.insert(task_id, callback);
        task_id
    }

    pub fn remove(&mut self, task_id: i32) -> () {
        self.table.remove(&task_id);
    }
}

struct Scheduler {
    // treat timers as tasks, No particular reason.
    timers: Arc<Mutex<HashMap<TimerId, SchedulingTrategy>>>,
    /// ready to run tasks
    expired: Arc<Mutex<VecDeque<i32>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        let mutexed_queue = std::sync::Mutex::new(VecDeque::new());
        Self {
            timers: Arc::new(Mutex::new(HashMap::new())),
            expired: Arc::new(mutexed_queue),
        }
    }

    pub fn fetch_expired_timer(&mut self) -> Option<i32> {
        let mut expired = self.expired.lock().unwrap();
        expired.pop_front()
    }

    pub fn forget(&mut self, task_id: i32) -> () {
        // we are not removing from expired container
        // will simply ignore if we could not find task in HashMap
        self.timers.lock().unwrap().remove(&task_id);
    }

    pub fn schedule(&mut self, task_id: i32, startegy: SchedulingTrategy) -> () {
        self.timers
            .lock()
            .unwrap()
            .insert(task_id, startegy.clone());
        let available = self.expired.clone();

        match startegy {
            SchedulingTrategy::Once(interval) => {
                std::thread::spawn(move || {
                    sleep(time::Duration::from_millis(interval.into()));
                    let mut expired = available.lock().unwrap();
                    expired.push_back(task_id);
                });
                // self.timers.lock().unwrap().remove(&task_id);
            }
            SchedulingTrategy::Periodic(interval) => {
                let timers_mutex = self.timers.clone();
                std::thread::spawn(move || {
                    sleep(time::Duration::from_millis(interval.into()));
                    let mut expired = available.lock().unwrap();
                    expired.push_back(task_id);
                    let mut timers = timers_mutex.lock().unwrap();

                    if timers.contains_key(&task_id) {
                        timers.insert(task_id, startegy.clone());
                    }
                });
            }
        }
    }
}
