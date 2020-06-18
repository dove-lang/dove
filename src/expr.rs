use crate::token::*;

pub enum Expr {
    Assign   (Token, Box<Expr>),
    Binary   (Box<Expr>, Token, Box<Expr>),
    Grouping (Box<Expr>),
    Literal  (Literals),
    Unary    (Token, Box<Expr>),
    Variable (Token),
}

pub trait Visitor {
    type Result;

    fn visit(&mut self, expr: &Expr) -> Self::Result;
}
