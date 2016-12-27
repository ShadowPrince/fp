use std::fmt;
use std::ops::Add;

use super::*;

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &Argument::String(ref s) => format!("{}", s),
            &Argument::Number(ref i) => format!("{}", i),
        })
    }
}

impl Add for Argument {
    type Output=Argument;

    fn add(self, b: Argument) -> Argument {
        match (self, b) {
            (Argument::String(mut a), Argument::String(b)) => {
                a.push_str(&b);

                Argument::String(a)
            },

            (Argument::Number(a), Argument::Number(b)) => {
                Argument::Number(a + b)
            }

            (Argument::Number(a), arg_b) => {
                Argument::String(a.to_string()) + arg_b
            },

            (arg_a, Argument::Number(b)) => {
                arg_a + Argument::String(b.to_string())
            },
        }
    }
}

impl From<String> for Argument {
    fn from(v: String) -> Self {
        Argument::String(v)
    }
}

impl From<i32> for Argument {
    fn from(v: i32) -> Self {
        Argument::Number(v as f32)
    }
}

impl From<f32> for Argument {
    fn from(v: f32) -> Self {
        Argument::Number(v)
    }
}

impl<'a> From<&'a Argument> for Argument {
    fn from(v: &'a Argument) -> Self {
        match v {
            &Argument::String(ref s) => Argument::String(s.to_owned()),
            &Argument::Number(ref s) => Argument::Number(*s),
        }
    }
}
