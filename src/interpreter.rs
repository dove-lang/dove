use crate::ast::*;
use crate::environment::Environment;
use crate::token::*;
use crate::ast_printer::AstPrinter;

pub struct Interpreter {
    environment: Box<Environment>
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter{ environment: Box::new(Environment::new(Option::None)) }
    }

    pub fn interpret(&mut self, exprs: Vec<Expr>) {
        let mut printer = AstPrinter{};
        for expr in exprs.iter() {
            println!("{} evaluates to literal: {:?}", printer.print(expr), self.evaluate(expr));
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Literals {
        self.visit(expr)
    }
}

impl ExprVisitor for Interpreter {
    type Result = Literals;

    fn visit(&mut self, expr: &Expr) -> Self::Result {
        match expr {
            Expr::Assign(name, value) => {
                let val = self.evaluate(value);
                self.environment.assign(name.clone(), val.clone());
                val
            },
            Expr::Binary(left, operator, right) => {
                let left_val  = self.evaluate(left);
                let right_val = self.evaluate(right);

                match operator.token_type {
                    TokenType::GREATER => {
                        check_number_operand(operator, &left_val, &right_val);
                        Literals::Boolean(left_val.unwrap_number() > right_val.unwrap_number())
                    },
                    TokenType::GREATER_EQUAL => {
                        check_number_operand(operator, &left_val, &right_val);
                        Literals::Boolean(left_val.unwrap_number() >= right_val.unwrap_number())
                    },
                    TokenType::LESS => {
                        check_number_operand(operator, &left_val, &right_val);
                        Literals::Boolean(left_val.unwrap_number() < right_val.unwrap_number())
                    },
                    TokenType::LESS_EQUAL => {
                        check_number_operand(operator, &left_val, &right_val);
                        Literals::Boolean(left_val.unwrap_number() <= right_val.unwrap_number())
                    },
                    TokenType::BANG_EQUAL => Literals::Boolean(!is_equal(&left_val, &right_val)),
                    TokenType::EQUAL_EQUAL => Literals::Boolean(is_equal(&left_val, &right_val)),
                    TokenType::MINUS => {
                        check_number_operand(operator, &left_val, &right_val);
                        Literals::Number(left_val.unwrap_number() - right_val.unwrap_number())
                    },
                    TokenType::PLUS => {
                        match left_val {
                            Literals::Number(l) => { match right_val {
                                Literals::Number(r) => Literals::Number(l + r),
                                _ => panic!("Operands of <{}> must be two numbers or two strings.", operator.to_string())
                            }},
                            Literals::String(l) => { match right_val {
                                Literals::String(r) => Literals::String(format!("{}{}", l, r)),
                                _ => panic!("Operands of <{}> must be two numbers or two strings.", operator.to_string())
                            }},
                            _ => panic!("Operands of <{}> must be two numbers or two strings.", operator.to_string()),
                        }
                    },
                    TokenType::SLASH => {
                        check_number_operand(operator, &left_val, &right_val);
                        Literals::Number(left_val.unwrap_number() / right_val.unwrap_number())
                    },
                    TokenType::STAR => {
                        match left_val {
                            Literals::Number(l) => { match right_val {
                                Literals::Number(r) => Literals::Number(l * r),
                                Literals::String(r) => Literals::String(r.repeat(l as usize)),
                                _ => panic!("Operands of <{}> must be two numbers or a string and a number.", operator.to_string()),
                            }},
                            Literals::String(l) => { match right_val {
                                Literals::Number(r) => Literals::String(l.repeat(r as usize)),
                                _ => panic!("Operands of <{}> must be two numbers or a string and a number.", operator.to_string()),
                            }},
                            _ => panic!("Operands of <{}> must be two numbers or a string and a number.", operator.to_string()),
                        }
                    },
                    _ => panic!("Unsupported binary operator: <{}>", operator.to_string()),
                }
            },
            // TODO: Implement visit Call expression.
            Expr::Call(callee, paren, arguments) => {
                Literals::Nil
            },
            Expr::Grouping(expression) => {
                self.evaluate(expression)
            },
            // TODO: Implement visit If expression.
            Expr::IfExpr(condition, then_branch, else_branch) => {
                Literals::Nil
            },
            Expr::Literal(value) => {
                value.clone()
            },
            Expr::Unary(operator, right) => {
                let right_val = self.evaluate(right);

                match operator.token_type {
                    TokenType::BANG => Literals::Boolean(!is_truthy(&right_val)),
                    TokenType::MINUS => { match right_val {
                        Literals::Number(n) => Literals::Number(-n),
                        _ => panic!("Operands of <{}> must be a number.", operator.to_string()),
                    }},
                    _ => panic!("Unsupported unary operator: <{}>", operator.to_string()),
                }
            },
            Expr::Variable(name) => {
                self.environment.get(name).clone()
            },
        }
    }
}

//--- Helpers.
fn is_truthy(literal: &Literals) -> bool {
    match literal {
        Literals::Nil => false,
        Literals::Boolean(b) => *b,
        _ => true,
    }
}

fn is_equal(literal_a: &Literals, literal_b: &Literals) -> bool {
    match literal_a {
        Literals::String(s) => { match literal_b {
            Literals::String(other) => s == other,
            _ => false,
        }},
        Literals::Number(n) => { match literal_b {
            Literals::Number(other) => n == other,
            _ => false,
        }},
        Literals::Boolean(b) => { match literal_b {
            Literals::Boolean(other) => b == other,
            _ => false,
        }},
        Literals::Nil => { match literal_b {
            Literals::Nil => true,
            _ => false,
        }},
    }
}

fn check_number_operand(operator: &Token, left: &Literals, right: &Literals) {
    match left {
        Literals::Number(_) => { match right {
            Literals::Number(_) => return,
            _ => panic!("Operands of <{}> must be two numbers.", operator.to_string())
        }},
        _ => panic!("Operands of <{}> must be two numbers.", operator.to_string())
    }
}
