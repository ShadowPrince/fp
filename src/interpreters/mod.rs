use std::io::Result;

#[derive(Clone, Copy)]
pub enum ImportNamespace {
    Separate,
    Current,
}

#[derive(Clone, Copy)]
pub struct Environment {
    pub declaration_debug: bool,
}

pub type DecIdentifier = str;
pub trait Interpreter<'time> {
    fn init(&mut self, env: Environment);
    fn declare(&mut self, identifier: &DecIdentifier, number_of_arguments: usize, inline_code: &str) -> Result<()>;
    fn import(&mut self, description: &str, ns: ImportNamespace) -> Result<()>;

    fn pass_argument(&mut self, n: usize, argument: &Argument) -> Result<()>;
    fn evaluate(&mut self, id: &DecIdentifier, args_count: usize) -> Result<Box<Argument>>;
}

pub enum Argument {
    String(String),
    Number(f32),
}

pub mod args;
