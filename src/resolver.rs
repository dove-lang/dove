use std::collections::HashMap;

use crate::ast::{Expr, Stmt};
use crate::token::Token;
use crate::interpreter::Interpreter;
use crate::error_handler::CompiletimeErrorHandler;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum FunctionType {
    None,
    Function,
    Method,
    Initializer,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ClassType {
    None,
    Class,
    Subclass,
}

pub struct Resolver<'a> {
    scopes: Vec<HashMap<String, bool>>,
    interpreter: &'a mut Interpreter,
    error_handler: CompiletimeErrorHandler,
    current_function: FunctionType,
    current_class: ClassType,
    in_loop: bool,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Resolver<'a> {
        Resolver {
            scopes: vec![],
            interpreter,
            error_handler: CompiletimeErrorHandler::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
            in_loop: false,
        }
    }

    pub fn resolve(&mut self, statements: &'a Vec<Stmt>) {
        for statement in statements {
            self.visit_stmt(statement);
        }
    }
}

impl<'a> Resolver<'a> {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve(statements);
                self.end_scope();
            },
            Stmt::Break(token) => {
                if !self.in_loop {
                    self.error_handler.token_error(
                        token.clone(),
                        "Break statements can only be used inside loops.".to_string(),
                    );
                }
            },
            Stmt::Class(name, superclass, methods) => {
                self.declare(name);
                self.define(name);

                if let Some(superclass) = superclass {
                    self.resolve_local(superclass, &superclass.lexeme);

                    if superclass.lexeme == name.lexeme {
                        self.error_handler.token_error(
                            superclass.clone(),
                            "A class cannot inherit from itself.".to_string(),
                        );
                    }

                    // Begin scope to bind super
                    self.begin_scope();
                    self.scopes.last_mut().unwrap().insert("super".to_string(), true);
                }

                self.begin_scope();
                self.scopes.last_mut().unwrap().insert("self".to_string(), true);

                // Set class type
                let prev_class = self.current_class;
                self.current_class = if superclass.is_some() {
                    ClassType::Subclass
                } else {
                    ClassType::Class
                };

                for method in methods {
                    match method {
                        Stmt::Function(name, params, body) => self.visit_function(
                            params,
                            body,
                            if name.lexeme == "init"{
                                FunctionType::Initializer
                            } else {
                                FunctionType::Method
                            }),
                        _ => panic!("Class methods contain non-function statements."),
                    }
                }

                if superclass.is_some() {
                    // End scope that binds super
                    self.end_scope();
                }

                self.current_class = prev_class;

                self.end_scope();
            },
            Stmt::Continue(token) => {
                if !self.in_loop {
                    self.error_handler.token_error(
                        token.clone(),
                        "Continue statements can only be used inside loops.".to_string(),
                    );
                }
            },
            Stmt::Expression(expr) => {
                self.visit_expr(expr);
            },
            Stmt::For(variable, expr, block) => {
                self.visit_expr(expr);

                let prev_in_loop = self.in_loop;
                self.in_loop = true;

                self.begin_scope();
                self.declare(variable);
                self.define(variable);

                self.resolve(unwrap_block(block));

                self.end_scope();

                self.in_loop = prev_in_loop;
            },
            Stmt::Function(name, params, body) => {
                self.declare(name);
                self.define(name);

                self.visit_function(params, body, FunctionType::Function)
            },
            Stmt::Print(_, expr) => {
                self.visit_expr(expr);
            },
            Stmt::Return(token, expr) => {
                if self.current_function == FunctionType::None {
                    self.error_handler.token_error(
                        token.clone(),
                        "Cannot return from top-level code.".to_string(),
                    );
                }

                if let Some(expr) = expr {
                    if self.current_function == FunctionType::Initializer {
                        self.error_handler.token_error(
                            token.clone(),
                            "Cannot return a value from an initializer.".to_string(),
                        );
                    }

                    self.visit_expr(expr);
                }
            },
            Stmt::Variable(variable, initializer) => {
                self.declare(variable);

                if let Some(expr) = initializer {
                    self.visit_expr(expr);
                }

                self.define(variable);
            },
            Stmt::While(condition, block) => {
                self.visit_expr(condition);

                let prev_in_loop = self.in_loop;
                self.in_loop = true;

                self.visit_stmt(block);

                self.in_loop = prev_in_loop;
            },
        }
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        match expr {
            Expr::Array(exprs) => {
                for expr in exprs.iter() {
                    self.visit_expr(expr);
                }
            },
            Expr::Assign(variable, value) => {
                self.visit_expr(value);
                self.resolve_local(variable, &variable.lexeme)
            },
            Expr::Binary(expr1, _, expr2) => {
                self.visit_expr(expr1);
                self.visit_expr(expr2);
            },
            Expr::Call(callee, _, args) => {
                self.visit_expr(callee);

                for arg in args {
                    self.visit_expr(arg);
                }
            },
            Expr::Dictionary(exprs) => {
                for (key, value) in exprs {
                    self.visit_expr(key);
                    self.visit_expr(value);
                }
            },
            Expr::Get(obj, _) => {
                self.visit_expr(obj);
            },
            Expr::Grouping(expr) => {
                self.visit_expr(expr);
            },
            Expr::IfExpr(condition, then_branch, else_branch) => {
                self.visit_expr(condition);
                self.visit_stmt(then_branch);
                self.visit_stmt(else_branch);
            },
            Expr::IndexGet(expr, index) => {
                self.visit_expr(expr);
                self.visit_expr(index);
            },
            Expr::IndexSet(expr, index, value) => {
                self.visit_expr(expr);
                self.visit_expr(index);
                self.visit_expr(value);
            },
            Expr::Lambda(params, body) => {
                self.visit_function(params, body, FunctionType::Function)
            },
            Expr::Literal(_) => (),
            Expr::SelfExpr(token) => {
                if self.current_class == ClassType::None {
                    self.error_handler.token_error(
                        token.clone(),
                        "Cannot use 'self' outside of a class.".to_string(),
                    );
                }

                self.resolve_local(&token, &token.lexeme);
            },
            Expr::Set(obj, _, value) => {
                self.visit_expr(obj);
                self.visit_expr(value);
            },
            Expr::SuperExpr(token, _) => {
                if self.current_class == ClassType::None {
                    self.error_handler.token_error(
                        token.clone(),
                        "Cannot use 'super' outside of a class.".to_string(),
                    );
                } else if self.current_class == ClassType::Class {
                    self.error_handler.token_error(
                        token.clone(),
                        "Cannot use 'super' inside a class with no superclass.".to_string(),
                    );
                }

                self.resolve_local(token, &token.lexeme);
            },
            Expr::Tuple(exprs) => {
                for expr in exprs {
                    self.visit_expr(expr);
                }
            },
            Expr::Unary(_, expr) => {
                self.visit_expr(expr);
            },
            Expr::Variable(variable) => {
                if let Some(false) = self.get(&variable.lexeme) {
                    // Since declared but not defined, must be in variable initializer
                    self.error_handler.token_error(
                        variable.clone(),
                        "Cannot use a variable in its own initializer.".to_string(),
                    );
                } else {
                    self.resolve_local(variable, &variable.lexeme);
                }
            },
        }
    }

    fn visit_function(&mut self, params: &Vec<Token>, body: &'a Stmt, function_type: FunctionType) {
        let enclosing_function = self.current_function;
        self.current_function = function_type;

        // Set in loop to false to disallow top level break/continue in functions
        let prev_in_loop = self.in_loop;
        self.in_loop = false;

        self.begin_scope();

        for param in params {
            self.declare(param);
            self.define(param);
        }

        // We don't directly visit the block since we already created a new scope here with params
        self.resolve(unwrap_block(body));
        self.end_scope();

        self.in_loop = prev_in_loop;
        self.current_function = enclosing_function;
    }
}

impl<'a> Resolver<'a> {
    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, token: &Token) {
        let name = &token.lexeme;

        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name) {
                self.error_handler.token_error(
                    token.clone(),
                    "Variable with this name already declared in this scope.".to_string(),
                );
            } else {
                scope.insert(name.clone(), false);
            }
        }
    }

    fn define(&mut self, token: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(token.lexeme.clone(), true);
        }
    }

    fn get(&mut self, name: &String) -> Option<&bool> {
        match self.scopes.last() {
            Some(scope) => scope.get(name),
            None => None,
        }
    }

    // Resolve the expression as a local variable
    fn resolve_local(&mut self, token: &'a Token, name: &String) {
        for (depth, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.interpreter.resolve(token, depth);
                return;
            }
        }

        // Not found, assume it is global
    }

}

/// Utility function to unwrap a block into a vector of statements.
fn unwrap_block(block: &Stmt) -> &Vec<Stmt> {
    match block {
        Stmt::Block(statements) => statements,
        _ => panic!(),
    }
}
