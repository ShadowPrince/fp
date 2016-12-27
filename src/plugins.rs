use super::interpreters::*;

use std::io::Result;

extern crate libloading as libloading;
use self::libloading::{Library, Symbol};

const PLUGIN_INIT: &'static [u8] = b"init";
const PLUGIN_DECLARE: &'static [u8] = b"declare";
const PLUGIN_IMPORT: &'static [u8] = b"import";
const PLUGIN_PASSARGUMENT: &'static [u8] = b"pass_argument";
const PLUGIN_EVALUATE: &'static [u8] = b"evaluate";

pub type PluginInitFn = unsafe extern fn(Environment);
pub type PluginDeclareFn = unsafe extern fn(identifier: &DecIdentifier, args: usize, code: &str) -> Result<()>;
pub type PluginImportFn = fn(descr: &str, ns: ImportNamespace) -> Result<()>;
pub type PluginPassArgumentFn = fn(n: usize, arg: &Argument) -> Result<()>;
pub type PluginEvaluateFn = fn(id: &DecIdentifier, args: usize) -> Result<Box<Argument>>;

pub fn load(path: &str) -> Library {
    Library::new(path).unwrap()
}

impl<'a> Interpreter<'a> for Library {
    fn init(&mut self, env: Environment) {
        unsafe {
            self.get::<Symbol<PluginInitFn>>(PLUGIN_INIT).unwrap()(env)
        }
    }

    fn declare(&mut self, id: &DecIdentifier, args: usize, code: &str) -> Result<()> {
        unsafe {
            self.get::<Symbol<PluginDeclareFn>>(PLUGIN_DECLARE).unwrap()(id, args, code)
        }
    }

    fn import(&mut self, descr: &str, ns: ImportNamespace) -> Result<()> {
        unsafe {
            self.get::<Symbol<PluginImportFn>>(PLUGIN_IMPORT).unwrap()(descr, ns)
        }
    }

    fn pass_argument(&mut self, n: usize, arg: &Argument) -> Result<()> {
        unsafe {
            self.get::<Symbol<PluginPassArgumentFn>>(PLUGIN_PASSARGUMENT).unwrap()(n, arg)
        }
    }

    fn evaluate(&mut self, id: &DecIdentifier, args: usize) -> Result<Box<Argument>> {
        unsafe {
            self.get::<Symbol<PluginEvaluateFn>>(PLUGIN_EVALUATE).unwrap()(id, args)
        }
    }
}
