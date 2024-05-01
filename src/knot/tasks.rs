use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub enum Task {
    Once {
        timeout: u32,
        callback: TaskId,
    },
    Periodic {
        interval: u32,
        callback: TaskId,
    },
    Script {
        source: String,
    },
    CallBack {
        value: v8::Global<v8::Value>,
        args: Vec<v8::Global<v8::Value>>,
    },
}

pub type TaskId = i32;

pub struct TasksTable {
    pub(crate) table: HashMap<TaskId, Task>,
    next_id: i32,
}

impl TasksTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            next_id: 0,
        }
    }

    fn task_id(&mut self) -> i32 {
        self.next_id += 1;
        self.next_id
    }

    pub fn as_mut(&mut self, id: &TaskId) -> Option<&mut Task> {
        self.table.get_mut(id)
    }

    pub fn register(&mut self, task: Task) -> i32 {
        let task_id = self.task_id();
        self.table.insert(task_id, task);
        task_id
    }

    pub fn unregister(&mut self, task_id: &TaskId) -> Option<Task> {
        self.table.remove(task_id)
    }
}

pub struct TasksQueue<T = TaskId>(pub(crate) VecDeque<T>);

impl<T> TasksQueue<T> {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn enqueue(&mut self, item: T) -> () {
        self.0.push_back(item);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.0.pop_front()
    }
}
