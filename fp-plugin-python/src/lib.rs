extern crate fp;

use fp::interpreters::*;

use std::io::Result;
use std::io::{Error, ErrorKind};
use std::ops::Add;
use std::cell::RefCell;

pub extern crate cpython as api;

thread_local!(static PYTHON: RefCell<Python> = RefCell::new(Python::new(Environment {
    declaration_debug: false,
})));

#[no_mangle]
pub fn init(env: Environment) {
    PYTHON.with(|p| {
        p.borrow_mut().init(env);
    });
}

#[no_mangle]
pub fn declare(identifier: &DecIdentifier, args: usize, code: &str) -> Result<()> {
    PYTHON.with(|p| {
        p.borrow_mut().declare(identifier, args, code)
    })
}

#[no_mangle]
pub fn import(descr: &str, ns: ImportNamespace) -> Result<()> {
    PYTHON.with(|p| {
        p.borrow_mut().import(descr, ns)
    })
}

#[no_mangle]
pub fn pass_argument(n: usize, arg: &Argument) -> Result<()> {
    PYTHON.with(|p| {
        p.borrow_mut().pass_argument(n, arg)
    })
}

#[no_mangle]
pub fn evaluate(id: &DecIdentifier, args: usize) -> Result<Box<Argument>> {
    PYTHON.with(|p| {
        p.borrow_mut().evaluate(id, args)
    })
}

pub struct Python {
    gil: api::GILGuard,
    env: Environment,
    arguments: api::PyDict,
}

impl<'a> Python {
    fn new(env: Environment) -> Self {
        let gil = api::Python::acquire_gil();
        let pyargs = api::PyDict::new(gil.python());

        Python {
            gil: gil,
            arguments: pyargs,
            env: env,
        }
    }

    fn error_from_py(error: api::PyErr) -> Error {
        Error::new(ErrorKind::Other, &*format!("{:?}", error))
    }

    fn py(&self) -> api::Python {
        self.gil.python()
    }

    fn args_list(&self, n: usize) -> String {
        let mut result = String::new();
        let i = 'a' as u8;
        let to = i + n as u8;

        for i in i .. to {
            result.push(i as char);
            result.push_str(match i == to - 1 {
                true => "",
                false => ", ",
            })
        }

        result
    }

    fn args_names(&self) -> [&'static str; 6] {
        ["a", "b", "c", "d", "e", "f", ]
    }

    fn construct_function(&self, id: &DecIdentifier, args: usize, inline: &str) -> String {
        let mut code = String::new();

        code.push_str(&*format!("def {}(", id));
        code = code.add(&self.args_list(args));
        code.push_str(&*format!("):\n  {}", inline.replace("\n", "\n  ")));

        code
    }

    fn construct_lambda(&self, id: &DecIdentifier, args: usize, inline: &str) -> String {
        let mut code = String::new();

        code.push_str(&*format!("{} = lambda ", id));
        code = code.add(&self.args_list(args));
        code.push_str(&*format!(": {}", &inline[1..]));

        code
    }

    fn construct_declaration(&self, id: &DecIdentifier, args: usize, inline: &str) -> String {
        if inline.starts_with("\\") {
            self.construct_lambda(id, args, inline)
        } else {
            self.construct_function(id, args, inline)
        }
    }

    fn construct_call(&self, id: &DecIdentifier, args: usize) -> String {
        format!("return {}({})", id, self.args_list(args))
    }

    fn run(&self, code: &str) -> Result<()> {
        if self.env.declaration_debug {
            println!("{:?}", code);
        }

        match self.py().run(code, None, None) {
            Ok(_) => Ok(()),
            Err(e) => Err(Python::error_from_py(e)),
        }
    }

    fn eval(&self, code: &str) -> Result<api::PyObject> {
        match self.py().eval(code, None, Some(&self.arguments)) {
            Ok(object) => Ok(object),
            Err(e) => Err(Python::error_from_py(e)),
        }
    }
}

impl<'time> Interpreter<'time> for Python {
    fn init(&mut self, env: Environment) {
        self.env = env;
    }

    fn declare(&mut self, identifier: &DecIdentifier, args: usize, inline_code: &str) -> Result<()> {
        let wrapper_id = &format!("{}_wrapped", identifier);
        let code = self.construct_declaration(wrapper_id, args, inline_code);
        let wrapper_code = self.construct_declaration(identifier, args, &self.construct_call(wrapper_id, args));

        // panic if wrapper declaration failed
        let _ = self.run(&wrapper_code).unwrap();
        self.run(&code)
    }

    fn pass_argument(&mut self, n: usize, value: &Argument) -> Result<()> {
        let name = self.args_names()[n];

        let result = match value {
            &Argument::String(ref s) => self.arguments.set_item(self.py(), name, s),
            &Argument::Number(ref i) => self.arguments.set_item(self.py(), name, i),
        };

        match result {
            Err(e) => Err(Python::error_from_py(e)),
            _ => Ok(()),
        }
    }

    fn evaluate(&mut self, id: &DecIdentifier, args_count: usize) -> Result<Box<Argument>> {
        let code = format!("{}({})", id, self.args_list(args_count));
        match self.eval(&code) {
            Ok(object) => {
                if let Ok(string) = object.extract::<String>(self.py()) {
                    return Ok(Box::new(Argument::from(string)))
                }

                if let Ok(int) = object.extract::<i32>(self.py()) {
                    return Ok(Box::new(Argument::from(int)))
                }

                Err(Error::new(ErrorKind::Other, "unsupported result type"))
            },

            Err(e) => Err(e)
        }
    }

    fn import(&mut self, feature: &str, ns: ImportNamespace) -> Result<()> {
        self.run(&match ns {
            ImportNamespace::Current => format!("from {} import *", feature),
            ImportNamespace::Separate => format!("import {}", feature),
        })
    }
}
