use crate::token::{Token, Literals};
use crate::ast::Stmt;

#[derive(Debug)]
pub enum Expr {
    Assign   (Token, Box<Expr>),
    Binary   (Box<Expr>, Token, Box<Expr>),
    Call     (Box<Expr>, Token, Vec<Expr>),
    Grouping (Box<Expr>),
    IfExpr   (Box<Expr>, Box<Stmt>, Box<Stmt>),
    Literal  (Literals),
    Unary    (Token, Box<Expr>),
    Variable (Token),
}
