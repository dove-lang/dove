use crate::ast::{Expr, Stmt};

pub trait ExprVisitor {
    type Result;

    fn visit(&mut self, expr: &Expr) -> Self::Result;
}

pub trait StmtVisitor {
    type Result;

    fn visit(&mut self, stmt: &Stmt) -> Self::Result;
}