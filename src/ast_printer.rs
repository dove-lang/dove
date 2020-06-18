use crate::expr::*;

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&mut self, expr: &Expr) -> String {
        self.visit(expr)
    }

    fn parenthesize(&mut self, name: &String, exprs: Box<[&Expr]>) -> String {
        let mut res = "(".to_string();
        res.push_str(name);

        for expr in exprs.iter() {
            res.push_str(" ");
            res.push_str(&self.visit(expr));
        }
        res.push_str(" )");

        res
    }
}

impl Visitor for AstPrinter {
    type Result = String;

    fn visit(&mut self, expr: &Expr) -> Self::Result {
        match expr {
            Expr::Assign(name, value) => {
                format!("{} = {}", name.lexeme, self.visit(value))
            }
            Expr::Binary(left, operator, right) => {
                self.parenthesize(&operator.lexeme, Box::new([left, right]))
            }
            Expr::Grouping(expression) => {
                self.parenthesize(&"group".to_string(), Box::new([expression]))
            }
            Expr::Literal(value) => {
                format!("{:?}", value)
            }
            Expr::Unary(operator, right) => {
                self.parenthesize(&operator.lexeme, Box::new([right]))
            }
            Expr::Variable(name) => {
                name.lexeme.clone()
            }
        }
    }
}
