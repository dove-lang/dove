use crate::ast::Expr;
use crate::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Block       (Vec<Stmt>),
    Break,
    Class       (Token, Option<Token>, Vec<Stmt>),
    Expression  (Expr),
    For         (Token, Token, Box<Stmt>),
    Function    (Token, Vec<Token>, Box<Stmt>),
    Print       (Expr),
    Return      (Expr),
    Variable    (Token, Option<Expr>),
    While       (Expr, Box<Stmt>),
}
