use crate::token::*;

pub trait Expr {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result
        where Self: Sized;
}

pub trait Visitor {
    type Result;

    fn visit_assign_expr(&mut self, assign: &Assign) -> Self::Result;
    fn visit_binary_expr(&mut self, binary: &Binary) -> Self::Result;
    fn visit_grouping_expr(&mut self, grouping: &Grouping) -> Self::Result;
    fn visit_literal_expr(&mut self, literal: &Literal) -> Self::Result;
    fn visit_unary_expr(&mut self, unary: &Unary) -> Self::Result;
    fn visit_variable_expr(&mut self, variable: &Variable) -> Self::Result;
}

pub struct Assign {
    pub name: Token,
    pub value: Box<dyn Expr>,
}

impl Expr for Assign {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_assign_expr(self)
    }
}

pub struct Binary {
    pub left: Box<dyn Expr>,
    pub operator: Token,
    pub right: Box<dyn Expr>,
}

impl Expr for Binary {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_binary_expr(self)
    }
}

pub struct Grouping {
    pub expression: Box<dyn Expr>,
}

impl Expr for Grouping {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_grouping_expr(self)
    }
}

pub struct Literal {
    pub value: Literals,
}

impl Expr for Literal {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_literal_expr(self)
    }
}

pub struct Unary {
    pub operator: Token,
    pub right: Box<dyn Expr>,
}

impl Expr for Unary {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_unary_expr(self)
    }
}

pub struct Variable {
    pub name: Token,
}

impl Expr for Variable {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_variable_expr(self)
    }
}

