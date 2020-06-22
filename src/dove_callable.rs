use std::rc::Rc;
use std::cell::RefCell;

use crate::interpreter::Interpreter;
use crate::environment::Environment;
use crate::token::Literals;
use crate::ast::*;


pub trait DoveCallable {
    fn call(&mut self, interpreter: &mut Interpreter, argument_vals: &Vec<Literals>) -> Literals;
}

pub struct DoveFunction {
    declaration: Stmt,
}

impl DoveFunction {
    pub fn new(declaration: Stmt) -> DoveFunction {
        DoveFunction {
            declaration,
        }
    }

    pub fn arity(&self) -> usize {
        match &self.declaration {
            Stmt::Function(name_, params, body_) => params.len(),
            _ => { panic!("Cannot check arity. "); }
        }
    }
}

impl DoveCallable for DoveFunction {
    fn call(&mut self, interpreter: &mut Interpreter, argument_vals: &Vec<Literals>) -> Literals {
        let mut environment = Environment::new(Some(interpreter.globals.clone()));

        match &self.declaration {
            Stmt::Function(name, params, body) => {
                for i in 0..params.len() {
                    environment.define(params[i].clone(), argument_vals[i].clone());
                }
                interpreter.execute_block(&vec![*body.clone()], environment);

                Literals::Nil
            },
            _ => { panic!("Not callable. "); }
        }
    }
}
