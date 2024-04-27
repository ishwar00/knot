use core::panic;
use std::collections::HashMap;

use crate::knot::Knot;
use v8;

mod knot;
// mod process;

fn parse_args() -> (HashMap<String, String>, String) {
    use std::env;

    let args: Vec<String> = env::args().collect();
    let mut options = HashMap::new();
    let mut file = String::new();

    for arg in &args {
        if let Some(pos) = arg.find('=') {
            let (key, value) = arg.split_at(pos);
            let value = &value[1..];
            options.insert(key.into(), value.into());
        } else {
            file = arg.into();
        }
    }

    (options, file)
}

fn main() -> () {
    let mut isolate = Knot::init_v8();
    // TODO: set v8 flags
    isolate.set_microtasks_policy(v8::MicrotasksPolicy::Explicit);
    // TODO: add promise reject callback

    let mut handle_scope = v8::HandleScope::new(&mut isolate);
    let mut knot = Knot::new(&mut handle_scope);
    let (_, file) = parse_args();

    let source = std::fs::read_to_string(&file)
        .unwrap_or_else(|err| panic!("Failed to open {}: {}", file, err));

    let _ = knot.execute_script(source);
    knot.run_microtasks();
    knot.run_tasks();
}
