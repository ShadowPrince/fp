use std::env;
use std::io;

mod args;
mod tokenizer;
mod interpreters;

use interpreters::Interpreter;

fn process_function<'a, T, S>(args: &args::Arguments, stream: &mut tokenizer::Stream<S>, interpreter: &mut T)
    where T: Interpreter<'a>, S: io::Read
{
    let declaration_result = match &args.function {
        &args::Function::Map { code: ref code } => {
            interpreter.declare("foo", 1, &code)
        },

        &args::Function::MapIndexed { code: ref code } => {
            interpreter.declare("foo", 2, &code)
        },
    };

    match declaration_result {
        Ok(()) => {},
        Err(e) => panic!(e),
    }

    for (i, bytes) in stream.enumerate() {
        let string = String::from_utf8(bytes).unwrap();

        let result = match &args.function {
            &args::Function::Map { code: _ } => {
                interpreter.pass_argument(0, &string);

                interpreter.evaluate("foo", 1)
            },
            &args::Function::MapIndexed { code: _ } => {
                interpreter.pass_argument(0, i);
                interpreter.pass_argument(1, &string);

                interpreter.evaluate("foo", 2)
            },
        };

        match result {
            Ok(result) => {
                let resulting_string = String::from_utf8(*result).unwrap();
                println!("{}", resulting_string);
            },

            Err(_) if args.passtrough_on_error => {
                println!("{}", string);
            },

            _ => {},
        };
    }
}

fn main() {
    let args = args::parse_arguments().unwrap();

    let mut stream = tokenizer::Stream::new(&args.separator, io::stdin());
    let mut interpreter = interpreters::python::Python::new(&args.env);
    process_function(&args, &mut stream, &mut interpreter);

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
