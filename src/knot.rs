use core::panic;
use std::sync::Once;
use v8;
mod ops;

const KNOT_INIT: Once = Once::new();

#[allow(dead_code)]
pub struct Knot<'a, 'b> {
    context: v8::Local<'a, v8::Context>,
    context_scope: v8::ContextScope<'b, v8::HandleScope<'a>>,
}

type V8Instance = v8::OwnedIsolate;

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

    pub fn new(handle_scope: &'b mut v8::HandleScope<'a, ()>) -> Self {
        let global_template = Knot::create_glob_template(handle_scope);

        let knot_template = v8::ObjectTemplate::new(handle_scope);
        knot_template.set(
            v8::String::new(handle_scope, "Knot").unwrap().into(),
            global_template.into(),
        );

        let context = v8::Context::new_from_template(handle_scope, knot_template);
        let context_scope = v8::ContextScope::new(handle_scope, context);

        Self {
            context_scope,
            context,
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

        global
    }
}
