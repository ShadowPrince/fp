use super::*;

use std;
use std::io::Result;
use std::io::{Error, ErrorKind};

pub extern crate hlua as api;

impl ManualInto<Error> for api::LuaError {
    fn manual_into(&self) -> Error {
        Error::new(ErrorKind::Other, format!("{:?}", self))
    }
}

impl ManualInto<Vec<u8>> for String {
    fn manual_into(&self) -> Vec<u8> {
        Vec::from(self.as_bytes())
    }
}

impl<T: std::marker::Copy> ManualInto<Result<T>> for std::result::Result<T, api::LuaError> {
    fn manual_into(&self) -> Result<T> {
        match self {
            &Ok(ref value) => Ok(*value),
            &Err(ref e) => Err(e.manual_into()),
        }
    }
}

impl ManualInto<Vec<u8>> for i32 {
    fn manual_into(&self) -> Vec<u8> {
        Vec::from(format!("{}", self).as_bytes())
    }
}

pub struct Lua<'time> {
    i: api::Lua<'time>,
}

impl<'time> Lua<'time> {
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

    fn read_wrapper_var(&mut self) -> Vec<u8> {
        let code = "_wrapper_variable";

        match self.i.get::<String, _>(code) {
            Some(value) => return value.manual_into(),
            _ => {},
        }

        match self.i.get::<i32, _>(code) {
            Some(value) => return value.manual_into(),
            _ => {},
        }

        "".to_string().manual_into()
    }

    fn execute_to_wrapper_var(&mut self, code: &str) -> Result<()> {
        self.i.execute(&*format!("_wrapper_variable = {}", code)).manual_into()
    }
}

impl<'time> Interpreter<'time> for Lua<'time> {
    fn new() -> Self {
        let mut i = api::Lua::new();

        Lua {
            i: i,
        }
    }

    fn declare(&mut self, id: &DecIdentifier, args: usize, code: &str) -> Result<()> {
        let code = format!("function {}({}) {} end", id, self.args_list(args), code);
        self.i.execute(&code).manual_into()
    }

    fn evaluate<T>(&mut self, id: &DecIdentifier, args: &[T]) -> Result<Box<Vec<u8>>>
        where 
            for<'a> T: super::lua::api::Push<&'a mut super::lua::api::Lua<'time>>
            + super::python::api::ToPyObject
            + std::marker::Copy {
        for i in 0..args.len() {
            let name = self.args_names()[i];
            self.i.set(name, args[i]);
        }

        let code = format!("{}({})", id, self.args_list(args.len()));
        match self.execute_to_wrapper_var(&code) {
            Err(e) => return Err(e),
            Ok(_) => {},
        }

        Ok(Box::new(self.read_wrapper_var()))
    }

    fn import(&mut self, feature: &str, ns: ImportNamespace) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "not implemented"))
    }
}
