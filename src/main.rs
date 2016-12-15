use std::env;
use std::io;

mod tokenizer;
mod interpreters;

use interpreters::Interpreter;

fn do_work<T: Interpreter>(mut interpreter: T) {
    interpreter.declare("foo", 1, &*std::env::args().nth(2).unwrap()).unwrap();

    let stream = tokenizer::Stream::new("\n", io::stdin());
    for bytes in stream {
        let string = String::from_utf8(bytes).unwrap();
        let string_ref = string.as_str();
        match interpreter.evaluate("foo", &[string_ref]) {
            Ok(result) => {
                let resulting_string = String::from_utf8(*result).unwrap();
                println!("{}", resulting_string);
            },
            Err(e) => {
                println!("{}", string);
            }
        }

    }
}

fn main() {
    /*
    println!("{:?}", lua.declare("foo", 2, "return a + b"));
    let args = [1, 2 ];
    println!("{:?}", lua.evaluate("foo", &args));
    */

    //let mut interpreter = interpreters::lua::Lua::new();
    match &*std::env::args().nth(1).unwrap() {
        "lua" => {
            let mut interpreter = interpreters::lua::Lua::new();
            do_work(interpreter);
        },
        "python" => {
            let mut interpreter = interpreters::python::Python::new();
            do_work(interpreter);
        },
        _ => {},
    }

}
