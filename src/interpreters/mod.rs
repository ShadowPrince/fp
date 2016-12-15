use std;
use std::io::Result;
use std::io::{Error, ErrorKind};

pub enum ImportNamespace {
    Separate,
    Current,
}

pub type DecIdentifier = str;
pub trait Interpreter<'time> {
    fn new() -> Self;
    fn declare(&mut self, identifier: &DecIdentifier, number_of_arguments: usize, inline_code: &str) -> Result<()>;
    fn evaluate<T>(&mut self, id: &DecIdentifier, args: &[T]) -> Result<Box<Vec<u8>>>
        where 
            for<'a> T: self::lua::api::Push<&'a mut self::lua::api::Lua<'time>>
            + self::python::api::ToPyObject
            + std::marker::Copy;

    fn import(&mut self, description: &str, ns: ImportNamespace) -> Result<()>;
}

// used for some explicit conversions
pub trait ManualInto<T> {
    fn manual_into(&self) -> T;
}

pub mod python;
pub mod lua;
