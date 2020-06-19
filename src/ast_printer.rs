use crate::ast::*;

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&mut self, expr: &Expr) -> String {
        self.visit_expr(expr)
    }

    fn parenthesize(&mut self, name: &String, exprs: Vec<&Expr>) -> String {
        let mut res = "(".to_string();
        res.push_str(name);

        for expr in exprs.iter() {
            res.push_str(" ");
            res.push_str(&self.visit_expr(expr));
        }
        res.push_str(" )");

        res
    }
}

impl Visitor for AstPrinter {
    type Result = String;

    fn visit_expr(&mut self, expr: &Expr) -> Self::Result {
        match expr {
            Expr::Assign(name, value) => {
                format!("{} = {}", name.lexeme, self.visit_expr(value))
            }
            Expr::Binary(left, operator, right) => {
                self.parenthesize(&operator.lexeme, vec![left, right])
            }
            Expr::Grouping(expression) => {
                self.parenthesize(&"group".to_string(), vec![expression])
            }
            Expr::Literal(value) => {
                format!("{:?}", value)
            }
            Expr::Unary(operator, right) => {
                self.parenthesize(&operator.lexeme, vec![right])
            }
            Expr::Variable(name) => {
                name.lexeme.clone()
            }
            _ => "not implemented".to_string()
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> Self::Result {
        panic!()
    }
}
