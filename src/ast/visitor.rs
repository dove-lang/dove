use crate::ast::{Expr, Stmt};

pub trait ExprVisitor {
    type Result;

    fn visit_expr(&mut self, expr: &Expr) -> Result<Self::Result, ()>;
}

pub trait StmtVisitor {
    fn visit_stmt(&mut self, stmt: &Stmt);
}