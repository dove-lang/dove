use std::rc::Rc;
use std::cell::RefCell;

use crate::interpreter::{Interpreter, Interrupt};
use crate::environment::Environment;
use crate::token::{Token, Literals};
use crate::ast::*;
use crate::dove_class::DoveInstance;
use crate::constants::keywords;

pub trait DoveCallable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, argument_vals: &Vec<Literals>) -> Literals;
}

#[derive(Debug)]
pub struct DoveFunction {
    // pub declaration: Stmt,
    pub params: Vec<Token>,
    body: Stmt,
    // TODO: is Weak required here to prevent memory retain cycle?
    closure: Rc<RefCell<Environment>>,
}

impl DoveFunction {
    pub fn new(params: Vec<Token>, body: Stmt, closure: Rc<RefCell<Environment>>) -> DoveFunction {
        DoveFunction {
            params,
            body,
            closure,
        }
    }

    /// Create a new function that is enclosed by a scope containing local `self` referencing `instance`.
    pub fn bind(&self, instance: Rc<RefCell<DoveInstance>>) -> DoveFunction {
        let mut environment = Environment::new(Some(Rc::clone(&self.closure)));
        environment.define(keywords::SELF.to_string(), Literals::Instance(instance));
        DoveFunction::new(self.params.clone(), self.body.clone(), Rc::new(RefCell::new(environment)))
    }
}

impl DoveCallable for DoveFunction {
    fn call(&self, interpreter: &mut Interpreter, argument_vals: &Vec<Literals>) -> Literals {
        let mut environment = Environment::new(Some(self.closure.clone()));

        for i in 0..self.params.len() {
            environment.define(self.params[i].lexeme.clone(), argument_vals[i].clone());
        }

        let statements = match &self.body {
            Stmt::Block(statements) => statements,
            _ => panic!("Function have non-block body"),
        };

        match interpreter.execute_implicit_return(statements, environment) {
            Ok(implicit_return_val) => implicit_return_val,
            Err(Interrupt::Return(return_val)) => return_val,
            _ => Literals::Nil,
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}

pub struct BuiltinFunction<F>
where
    F: Fn(&Vec<Literals>) -> Literals
{
    arity: usize,
    function: F,
}

impl<F> BuiltinFunction<F>
where
    F: Fn(&Vec<Literals>) -> Literals
{
    pub fn new(arity: usize, function: F) -> BuiltinFunction<F> {
        BuiltinFunction {
            arity,
            function,
        }
    }
}

impl<F> DoveCallable for BuiltinFunction<F>
where
    F: Fn(&Vec<Literals>) -> Literals
{
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, _: &mut Interpreter, argument_vals: &Vec<Literals>) -> Literals {
        let f = &self.function;
        f(argument_vals)
    }
}
