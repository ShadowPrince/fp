use std::env;
use std::io;
use std::mem;

pub mod args;

mod tokenizer;
mod interpreters;
mod plugins;

use interpreters::Interpreter;
use interpreters::Argument;

fn process_function<'a, T, S>(args: &args::Arguments, stream: &mut tokenizer::Stream<S>, interpreter: &mut T)
    where T: Interpreter<'a>, S: io::Read
{
    let mut accumulator = Argument::Number(0.0);

    let declaration_result = match &args.function {
        &args::Function::Map { code: ref code } => {
            interpreter.declare("foo", 1, &code)
        },

        &args::Function::MapIndexed { code: ref code } => {
            interpreter.declare("foo", 2, &code)
        },

        &args::Function::Fold { code: ref code, value: ref value } => {
            accumulator = Argument::from(value);
            interpreter.declare("foo", 2, &code)
        },
    };

    match declaration_result {
        Ok(()) => {},
        Err(e) => panic!(e),
    }

    for (i, bytes) in stream.enumerate() {
        let input_arg = Argument::from(String::from_utf8(bytes).unwrap());

        let result = match &args.function {
            &args::Function::Map { code: _ } => {
                interpreter.pass_argument(0, &input_arg);

                interpreter.evaluate("foo", 1)
            },
            &args::Function::MapIndexed { code: _ } => {
                interpreter.pass_argument(0, &Argument::from(i as i32));
                interpreter.pass_argument(1, &input_arg);

                interpreter.evaluate("foo", 2)
            },

            &args::Function::Fold { code: ref code, value: ref value } => {
                interpreter.pass_argument(0, &accumulator);
                interpreter.pass_argument(1, &input_arg);

                interpreter.evaluate("foo", 2)
            },
        };

        match result {
            Ok(result) => {
                match &args.function.name() {
                    &args::FunctionName::Fold => accumulator = *result,
                    _ => println!("{}", result),
                }
            },

            Err(_) if args.passtrough_on_error => {
                println!("{}", input_arg);
            },

            _ => {},
        };
    }

    match &args.function.name() {
        &args::FunctionName::Fold => println!("{}", accumulator),
        _ => (),
    }
}

fn main() {
    let args = args::parse_arguments().unwrap();
    let mut stream = tokenizer::Stream::new(&args.separator, io::stdin());

    let mut interpreter = plugins::load("/Users/sp/projects/rust/fp-python/target/debug/libfp_python.dylib");
    process_function(&args, &mut stream, &mut interpreter);

    println!("attempt to drop");
    mem::drop(interpreter);
    println!("main will now exit");
    /*
    if args.is_present("lua") {
        let mut interpreter = interpreters::lua::Lua::new();
        run(&mut stream, &mut interpreter, &args);
    } else {
    let mut interpreter = interpreters::python::Python::new();
    run(&mut stream, &mut interpreter, "map");
    */
    //}
}
