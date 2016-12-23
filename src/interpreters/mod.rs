use std;
use std::io::Result;

pub enum ImportNamespace {
    Separate,
    Current,
}

pub struct Environment {
    pub declaration_debug: bool,
}

pub type DecIdentifier = str;
pub trait Interpreter<'time> {
    fn new(env: &'time Environment) -> Self;
    fn declare(&mut self, identifier: &DecIdentifier, number_of_arguments: usize, inline_code: &str) -> Result<()>;
    fn import(&mut self, description: &str, ns: ImportNamespace) -> Result<()>;

    fn pass_argument<T>(&mut self, n: usize, argument: T) -> Result<()> where T: self::python::api::ToPyObject;
    fn evaluate(&mut self, id: &DecIdentifier, args_count: usize) -> Result<Box<Vec<u8>>>;
}

// used for some explicit conversions
pub trait ManualInto<T> {
    fn manual_into(&self) -> T;
}

pub mod python;
//pub mod lua;
