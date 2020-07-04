use crate::ast::Expr;
use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Stmt {
    Block       (Vec<Stmt>),
    Break       (Token),
    Continue    (Token),
    Class       (Token, Option<Token>, Vec<Stmt>),
    Expression  (Expr),
    For         (Token, Expr, Box<Stmt>),
    Function    (Token, Vec<Token>, Box<Stmt>),
    Print       (Token, Expr),
    Return      (Token, Option<Expr>),
    Variable    (Token, Option<Expr>),
    While       (Expr, Box<Stmt>),
}
