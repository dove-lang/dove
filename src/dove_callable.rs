use crate::interpreter::Interpreter;
use crate::environment::Environment;
use crate::token::Literals;
use crate::ast::*;
use std::rc::Rc;
use std::cell::RefCell;


pub trait DoveCallable {
    fn call(&mut self, interpreter: &mut Interpreter, argument_vals: &Vec<Literals>) -> Literals;
}

pub struct DoveFunction {
    declaration: Stmt,
    closure: Rc<RefCell<Environment>>,
}

impl DoveFunction {
    pub fn new(declaration: Stmt, closure: Rc<RefCell<Environment>>) -> DoveFunction {
        DoveFunction {
            declaration,
            closure,
        }
    }

    pub fn arity(&self) -> usize {
        match &self.declaration {
            Stmt::Function(_, params, _) => params.len(),
            _ => { panic!("Cannot check arity. "); }
        }
    }
}

impl DoveCallable for DoveFunction {
    fn call(&mut self, interpreter: &mut Interpreter, argument_vals: &Vec<Literals>) -> Literals {
        let mut environment = Environment::new(Some(self.closure.clone()));

        match &self.declaration {
            Stmt::Function(_, params, body) => {
                for i in 0..params.len() {
                    environment.define(params[i].clone(), argument_vals[i].clone());
                }

                let statements = match body.as_ref() {
                    Stmt::Block(statements) => statements,
                    _ => panic!("Function have non-block body"),
                };

                match interpreter.execute_block(statements, environment) {
                    Ok(_) => {},
                    Err(return_val) => {
                        return return_val;
                    }
                }

                Literals::Nil
            },
            _ => { panic!("Not callable. "); }
        }
    }
}
