use crate::ast::*;

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&mut self, expr: &Expr) -> String {
        self.visit_expr(expr).unwrap()
    }

    fn parenthesize(&mut self, name: &String, exprs: Vec<&Expr>) -> String {
        let mut res = "(".to_string();
        res.push_str(name);

        for expr in exprs.iter() {
            res.push_str(" ");
            res.push_str(&self.visit_expr(expr).unwrap());
        }
        res.push_str(" )");

        res
    }
}

impl ExprVisitor for AstPrinter {
    type Result = String;

    fn visit_expr(&mut self, expr: &Expr) -> Result<Self::Result, ()> {
        match expr {
            Expr::Assign(name, value) => {
                Ok(format!("{} = {}", name.lexeme, self.visit_expr(value).unwrap()))
            }
            Expr::Binary(left, operator, right) => {
                Ok(self.parenthesize(&operator.lexeme, vec![left, right]))
            }
            Expr::Grouping(expression) => {
                Ok(self.parenthesize(&"group".to_string(), vec![expression]))
            }
            Expr::Literal(value) => {
                Ok(format!("{:?}", value))
            }
            Expr::Unary(operator, right) => {
                Ok(self.parenthesize(&operator.lexeme, vec![right]))
            }
            Expr::Variable(name) => {
                Ok(name.lexeme.clone())
            }
            _ => Ok("not implemented (ExprVisitor for AstPrinter)".to_string())
        }
    }
}
