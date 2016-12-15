use super::*;

use std;
use std::io::Result;
use std::io::{Error, ErrorKind};
use std::ops::Add;
use std::convert::Into;

pub extern crate cpython as api;
use self::api::ToPyObject;

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

pub struct Python {
    gil: api::GILGuard,
}

impl Python {
    fn py(&self) -> api::Python {
        self.gil.python()
    }

    fn args_list(&self, n: usize) -> String {
        let mut result = String::new();
        let mut i = 'a' as u8;
        let mut to = i + n as u8;

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
        code.push_str(&*format!(": {}", inline));

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
        match self.py().run(code, None, None) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.manual_into()),
        }
    }

    fn eval(&self, code: &str, args: Option<&api::PyDict>) -> Result<api::PyObject> {
        match self.py().eval(code, None, args) {
            Ok(object) => Ok(object),
            Err(e) => Err(e.manual_into()),
        }
    }
}

impl Interpreter for Python {
    fn new() -> Self {
        let gil = api::Python::acquire_gil();

        Python {
            gil: gil,
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


    fn evaluate<T>(&mut self, id: &DecIdentifier, args: &[T]) -> Result<Box<Vec<u8>>>
        where for<'a> T: super::lua::api::LuaPush
            + super::python::api::ToPyObject
            + std::marker::Copy {
        let pyargs = api::PyDict::new(self.py());
        for i in 0..args.len() {
            let name = self.args_names()[i];
            match pyargs.set_item(self.py(), name, &args[0]) {
                Err(e) => { return Err(Error::new(ErrorKind::Other, "failed")); },
                _ => {},
            }
        }

        let code = format!("{}({})", id, self.args_list(args.len()));
        match self.eval(&code, Some(&pyargs)) {
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
