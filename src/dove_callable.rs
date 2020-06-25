use std::rc::Rc;
use std::cell::RefCell;

use crate::interpreter::{Interpreter, Interrupt};
use crate::environment::Environment;
use crate::token::Literals;
use crate::ast::*;
use crate::dove_class::DoveInstance;

pub trait DoveCallable {
    fn call(&self, interpreter: &mut Interpreter, argument_vals: &Vec<Literals>) -> Literals;
}

#[derive(Debug)]
pub struct DoveFunction {
    pub declaration: Stmt,
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

    /// Create a new function that is enclosed by a scope containing local `self` referencing `instance`.
    pub fn bind(&self, instance: Rc<RefCell<DoveInstance>>) -> DoveFunction {
        let mut environment = Environment::new(Some(Rc::clone(&self.closure)));
        environment.define("self".to_string(), Literals::Instance(instance));
        DoveFunction::new(self.declaration.clone(), Rc::new(RefCell::new(environment)))
    }
}

impl DoveCallable for DoveFunction {
    fn call(&self, interpreter: &mut Interpreter, argument_vals: &Vec<Literals>) -> Literals {
        let mut environment = Environment::new(Some(self.closure.clone()));

        match &self.declaration {
            Stmt::Function(_, params, body) => {
                for i in 0..params.len() {
                    environment.define(params[i].lexeme.clone(), argument_vals[i].clone());
                }

                let statements = match body.as_ref() {
                    Stmt::Block(statements) => statements,
                    _ => panic!("Function have non-block body"),
                };

                match interpreter.execute_block(statements, environment) {
                    Err(Interrupt::Return(return_val)) => return_val,
                    _ => Literals::Nil,
                }
            },
            _ => { panic!("Not callable. "); }
        }
    }
}

#[derive(Debug)]
pub struct DoveLambda {
    pub declaration: Expr,
    closure: Rc<RefCell<Environment>>,
}

impl DoveLambda {
    pub fn new(declaration: Expr, closure: Rc<RefCell<Environment>>) -> DoveLambda {
        DoveLambda {
            declaration,
            closure
        }
    }

    pub fn arity(&self) -> usize {
        match &self.declaration {
            Expr::Lambda(params, _) => params.len(),
            _ => { panic!("Cannot check arity. "); }
        }
    }
}

impl DoveCallable for DoveLambda {
    fn call(&self, interpreter: &mut Interpreter, argument_vals: &Vec<Literals>) -> Literals {
        let mut environment = Environment::new(Some(self.closure.clone()));

        match &self.declaration {
            Expr::Lambda(params, body) => {
                for i in 0..params.len() {
                    environment.define(params[i].lexeme.clone(), argument_vals[i].clone());
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
