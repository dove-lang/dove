use crate::interpreter::Interpreter;
use crate::token::Literals;

pub trait DoveCallable {
    fn call(&mut self, interpreter: &Interpreter, argument_vals: &Vec<Literals>) -> Literals;
}
