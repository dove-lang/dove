use crate::token::{Token, Literals};
use crate::ast::Stmt;

#[derive(Debug, Clone)]
pub enum Expr {
    Array      (Vec<Expr>),
    Assign     (Token, Box<Expr>),
    Binary     (Box<Expr>, Token, Box<Expr>),
    Call       (Box<Expr>, Token, Vec<Expr>),
    Dictionary (Vec<(Expr, Expr)>),
    Get        (Box<Expr>, Token),
    Grouping   (Box<Expr>),
    IfExpr     (Box<Expr>, Box<Stmt>, Box<Stmt>),
    IndexGet   (Box<Expr>, Box<Expr>),
    IndexSet   (Box<Expr>, Box<Expr>, Box<Expr>),
    Literal    (Literals),
    Set        (Box<Expr>, Token, Box<Expr>),
    SelfExpr   (Token),
    SuperExpr  (Token, Token),
    Tuple      (Vec<Expr>),
    Unary      (Token, Box<Expr>),
    Variable   (Token),
}
