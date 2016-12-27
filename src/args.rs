use std::env;
use std::io::Result;
use std::io::{Error, ErrorKind};

use super::interpreters::Environment;
use super::interpreters::Argument;

pub enum FunctionName {
    Map,
    MapIndexed,
    Fold,
}

pub enum Function {
    Map { code: String },
    MapIndexed { code: String },
    Fold { code: String, value: Argument },
}

impl Function {
    fn name_from_string(value: &str) -> Option<FunctionName> {
        match value {
            "map" => Some(FunctionName::Map),
            "map-indexed" => Some(FunctionName::MapIndexed),
            "fold" => Some(FunctionName::Fold),

            _ => None,
        }
    }

    pub fn name(&self) -> FunctionName {
        match self {
            &Function::Map { code: _ } => FunctionName::Map,
            &Function::MapIndexed { code: _ } => FunctionName::MapIndexed,
            &Function::Fold { code: _, value: _ } => FunctionName::Fold,
        }
    }

    fn parse_inline_code<T: Iterator<Item=String>>(slice: &mut T) -> String {
        let mut inline = String::new();
        for mut el in slice {
            el = el.replace("#", "\"");
            inline.push_str(&el);
            inline.push(' ');
        }

        inline
    }

    fn parse_value(s: &String) -> Argument {
        if let Ok(value) = s.parse::<i32>() {
            Argument::Number(value as f32)
        } else if let Ok(value) = s.parse::<f32>() {
            Argument::Number(value)
        } else {
            Argument::String(s.to_owned())
        }
    }

    fn parse_arguments<T: Iterator<Item=String>>(name: FunctionName, slice: &mut T) -> Option<Function> {
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

            FunctionName::Fold => {
                let initial = Function::parse_value(&slice.nth(0).unwrap());
                let inline = Function::parse_inline_code(&mut slice.skip(0));

                Some(Function::Fold {
                    code: inline,
                    value: initial,
                })
            }
        }
    }

}

impl Into<FunctionName> for Function {
    fn into(self) -> FunctionName {
        match self {
            Function::Map { code: _ } => FunctionName::Map,
            Function::MapIndexed { code: _ } => FunctionName::MapIndexed,
            Function::Fold { code: _, value: _ } => FunctionName::Fold,
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
