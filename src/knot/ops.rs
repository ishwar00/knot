use v8;

pub fn print(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut _retval: v8::ReturnValue,
) {
    for i in 0..args.length() {
        let mut arg_i = args
            .get(0)
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
