use super::*;

use std;
use std::io::Result;
use std::io::{Error, ErrorKind};
use std::ops::Add;

pub extern crate cpython as api;

struct PythonObject<'a> {
    object: &'a api::PyObject,
    python: &'a api::Python<'a>,
}

impl<'a> ManualInto<Vec<u8>> for PythonObject<'a> {
    fn manual_into(&self) -> Vec<u8> {
        if let Ok(string) = self.object.extract::<String>(*self.python) {
            return Vec::from(string.as_bytes());
        }

        if let Ok(int) = self.object.extract::<i32>(*self.python) {
            return Vec::from(format!("{}", int).as_bytes());
        }

        Vec::new()
    }
}

impl ManualInto<Error> for api::PyErr {
    fn manual_into(&self) -> Error {
        Error::new(ErrorKind::Other, &*format!("{:?}", self))
    }
}

pub struct Python<'a> {
    gil: api::GILGuard,
    env: &'a Environment,
    arguments: api::PyDict,
}

impl<'a> Python<'a> {
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
            Err(e) => Err(e.manual_into()),
        }
    }

    fn eval(&self, code: &str) -> Result<api::PyObject> {
        match self.py().eval(code, None, Some(&self.arguments)) {
            Ok(object) => Ok(object),
            Err(e) => Err(e.manual_into()),
        }
    }
}

impl<'time> Interpreter<'time> for Python<'time> {
    fn new(env: &'time Environment) -> Self {
        let gil = api::Python::acquire_gil();
        let pyargs = api::PyDict::new(gil.python());

        Python {
            gil: gil,
            arguments: pyargs,
            env: env,
        }
    }

    fn declare(&mut self, identifier: &DecIdentifier, args: usize, inline_code: &str) -> Result<()> {
        let wrapper_id = &format!("{}_wrapped", identifier);
        let code = self.construct_declaration(wrapper_id, args, inline_code);
        let wrapper_code = self.construct_declaration(identifier, args, &self.construct_call(wrapper_id, args));

        // panic if wrapper declaration failed
        let _ = self.run(&wrapper_code).unwrap();
        self.run(&code)
    }

    fn pass_argument<T>(&mut self, n: usize, value: T) -> Result<()> where T: api::ToPyObject {
        let name = self.args_names()[n];
        match self.arguments.set_item(self.py(), name, &value) {
            Err(_) => { return Err(Error::new(ErrorKind::Other, "failed")); }, //TODO
                _ => {},
        }

        Ok(())
    }

    fn evaluate(&mut self, id: &DecIdentifier, args_count: usize) -> Result<Box<Vec<u8>>> {
        let code = format!("{}({})", id, self.args_list(args_count));
        match self.eval(&code) {
            Ok(object) => {
                let wrapper = PythonObject {
                    object: &object,
                    python: &self.py(),
                };

                Ok(Box::new(wrapper.manual_into()))
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
