use crate::ast::{Expr, Stmt};

pub trait Visitor {
    type Result;

    fn visit_expr(&mut self, expr: &Expr) -> Self::Result;
    fn visit_stmt(&mut self, stmt: &Stmt) -> Self::Result;
}