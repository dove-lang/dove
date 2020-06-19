use crate::ast::Expr;
use crate::token::Token;

pub enum Stmt {
    Block       (Vec<Stmt>),
    Class       (Token, Option<Token>, Vec<Stmt>),
    Expression  (Expr),
    Function    (Token, Vec<Token>, Box<Stmt>),
    Print       (Expr),
    Return      (Expr),
    Variable    (Token, Option<Expr>),
    While       (Expr, Box<Stmt>),
}
