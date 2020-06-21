use crate::ast::Expr;
use crate::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Block       (Vec<Stmt>),
    Break,
    Continue,
    Class       (Token, Option<Token>, Vec<Stmt>),
    Expression  (Expr),
    For         (Token, Expr, Box<Stmt>),
    Function    (Token, Vec<Token>, Box<Stmt>),
    Print       (Expr),
    Return      (Expr),
    Variable    (Token, Option<Expr>),
    While       (Expr, Box<Stmt>),
}
