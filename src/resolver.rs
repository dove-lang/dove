use std::collections::HashMap;

use crate::ast::{Expr, Stmt};
use crate::token::Token;
use crate::interpreter::Interpreter;
use crate::error_handler::CompiletimeErrorHandler;

pub struct Resolver<'a> {
    scopes: Vec<HashMap<String, bool>>,
    interpreter: &'a mut Interpreter,
    error_handler: CompiletimeErrorHandler,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Resolver<'a> {
        Resolver {
            scopes: vec![],
            interpreter,
            error_handler: CompiletimeErrorHandler::new(),
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
                // TODO
            },
            Stmt::Class(name, superclass, methods) => {
                // TODO: after finishing class
            },
            Stmt::Continue(token) => {
                // TODO
            },
            Stmt::Expression(expr) => {
                self.visit_expr(expr);
            },
            Stmt::For(variable, expr, block) => {
                self.visit_expr(expr);

                self.begin_scope();
                self.declare(&variable.lexeme);
                self.define(&variable.lexeme);

                self.resolve(unwrap_block(block));

                self.end_scope();
            },
            Stmt::Function(name, params, body) => {
                self.declare(&name.lexeme);
                self.define(&name.lexeme);

                self.visit_function(params, body)
            },
            Stmt::Print(expr) => {
                self.visit_expr(expr);
            },
            Stmt::Return(expr) => {
                // TODO
                if let Some(expr) = expr {
                    self.visit_expr(expr);
                }
            },
            Stmt::Variable(variable, initializer) => {
                self.declare(&variable.lexeme);

                if let Some(expr) = initializer {
                    self.visit_expr(expr);
                }

                self.define(&variable.lexeme);
            },
            Stmt::While(condition, block) => {
                self.visit_expr(condition);
                self.visit_stmt(block);
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
                self.resolve_local(expr, &variable.lexeme)
            },
            Expr::Binary(expr1, _, expr2) => {
                self.visit_expr(expr1);
                self.visit_expr(expr2);
            },
            Expr::Call(callee, paren, args) => {
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
            Expr::Get(obj, token) => {
                self.visit_expr(obj);
                // TODO: token shouldn't need to be checked?
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
            Expr::Literal(_) => (),
            Expr::SelfExpr(token) => {
                // TODO: after finishing class
            },
            Expr::Set(obj, token, value) => {
                self.visit_expr(obj);
                self.visit_expr(value);
            },
            Expr::SuperExpr(token, method) => {
                // TODO: after finishing class
            },
            Expr::Tuple(exprs) => {
                for expr in exprs {
                    self.visit_expr(expr);
                }
            },
            Expr::Unary(token, expr) => {
                self.visit_expr(expr);
            },
            Expr::Variable(variable) => {
                if let Some(false) = self.get(&variable.lexeme) {
                    // Since declared but not defined, must be in variable initializer
                    self.error_handler.token_error(variable.clone(), "Cannot use a variable in its own initializer.".to_string());
                } else {
                    self.resolve_local(expr, &variable.lexeme)
                }
            },
        }
    }

    fn visit_function(&mut self, params: &Vec<Token>, body: &'a Stmt) {
        self.begin_scope();

        for param in params {
            self.declare(&param.lexeme);
            self.define(&param.lexeme);
        }

        // We don't directly visit the block since we already created a new scope here with params
        self.resolve(unwrap_block(body));

        self.end_scope();
    }
}

impl<'a> Resolver<'a> {
    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.clone(), false);
        }
    }

    fn define(&mut self, name: &String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.clone(), true);
        }
    }

    fn get(&mut self, name: &String) -> Option<&bool> {
        match self.scopes.last() {
            Some(scope) => scope.get(name),
            None => None,
        }
    }

    // Resolve the expression as a local variable
    fn resolve_local(&mut self, expr: &'a Expr, name: &String) {
        for (depth, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.interpreter.resolve(expr, depth);
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
