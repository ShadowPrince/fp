use std::env;
use std::io::Result;
use std::io::{Error, ErrorKind};

use super::interpreters::Environment;

pub enum FunctionName {
    Map,
    MapIndexed,
}

pub enum Function {
    Map { code: String },
    MapIndexed { code: String },
}

impl Function {
    fn name_from_string(value: &str) -> Option<FunctionName> {
        match value {
            "map" => Some(FunctionName::Map),
            "map-indexed" => Some(FunctionName::MapIndexed),
            _ => None,
        }
    }

    fn parse_inline_code(slice: &mut Iterator<Item=String>) -> String {
        let mut inline = String::new();
        for mut el in slice {
            el = el.replace("#", "\"");
            inline.push_str(&el);
            inline.push(' ');
        }

        inline
    }

    fn parse_arguments(name: FunctionName, slice: &mut Iterator<Item=String>) -> Option<Function> {
        match name {
            FunctionName::Map => {
                let inline = Function::parse_inline_code(slice);
                Some(Function::Map {
                    code: inline,
                })
            },

            FunctionName::MapIndexed => {
                let inline = Function::parse_inline_code(slice);
                Some(Function::MapIndexed {
                    code: inline,
                })
            },
        }
    }
}

pub struct Arguments {
    pub separator: String,
    pub function: Function,
    pub passtrough_on_error: bool,
    pub env: Environment,
}

pub fn parse_arguments() -> Result<Arguments> {
    let mut separator = "\n".to_string();
    let mut function = None;
    let mut declaration_debug = false;
    let mut passtrough_on_error = true;

    for (i, argument) in env::args().enumerate() {
        match Function::name_from_string(&argument) {
            Some(name) => {
                function = Function::parse_arguments(name, &mut env::args().skip(i + 1));
                break;
            },

            _ => {
                match argument.as_str() {
                    "-w" => separator = " ".to_string(),
                    "-d" => declaration_debug = true,
                    "-p" => passtrough_on_error = false,
                    _ => {},
                }
            },
        }
    }

    if let Some(function) = function {
        Ok(Arguments {
            separator: separator,
            function: function,
            passtrough_on_error: passtrough_on_error,
            env: Environment {
                declaration_debug: declaration_debug,
            }
        })
    } else {
        Err(Error::new(ErrorKind::Other, "foo")) // TODO
    }
}
