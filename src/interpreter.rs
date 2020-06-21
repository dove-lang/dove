use std::rc::Rc;
use std::cell::RefCell;

use crate::ast::*;
use crate::environment::Environment;
use crate::token::*;
use crate::error_handler::*;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    error_handler: RuntimeErrorHandler,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter{
            environment: Rc::new(RefCell::new(Environment::new(Option::None))),
            error_handler: RuntimeErrorHandler::new(),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) {
        for stmt in stmts.iter() {
            self.execute(stmt)
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Literals, ()> {
        self.visit_expr(expr)
    }

    fn execute(&mut self, stmt: &Stmt) {
        self.visit_stmt(stmt)
    }

    fn execute_block(&mut self, statements: &Vec<Stmt>, environment: Environment) {
        let previous = std::mem::replace(&mut self.environment, Rc::new(RefCell::new(environment)));

        for stmt in statements.iter() {
            self.execute(stmt);
        }

        self.environment = previous;
    }

    fn check_number_operand(&mut self, operator: &Token, left: &Literals, right: &Literals) -> Result<(), ()> {
        match left {
            Literals::Number(_) => { match right {
                Literals::Number(_) => Ok(()),
                _ => {
                    self.report_err(operator.clone(), format!("Operands of '{}' must be two numbers.", operator.lexeme));
                    Err(())
                }
            }},
            _ => {
                self.report_err(operator.clone(), format!("Operands of '{}' must be two numbers.", operator.lexeme));
                Err(())
            }
        }
    }

    fn report_err(&mut self, token: Token, message: String) {
        let rt_err = RuntimeError::new(token.clone(), message);
        self.error_handler.runtime_error(rt_err);
    }
}

impl ExprVisitor for Interpreter {
    type Result = Literals;

    fn visit_expr(&mut self, expr: &Expr) -> Result<Self::Result, ()> {
        match expr {
            Expr::Assign(name, value) => {
                let val = match self.evaluate(value) {
                    Ok(v) => v,
                    Err(()) => return Err(()),
                };
                let res = self.environment.borrow_mut().assign(name.clone(), val.clone());
                match res {
                    Ok(_) => Ok(val),
                    Err(_) => {
                        self.report_err(name.clone(), format!("Cannot assign value to '{}', as it is not found in scope.", name.lexeme));
                        Err(())
                    }
                }
            },

            Expr::Binary(left, operator, right) => {
                let left_val  = match self.evaluate(left) {
                    Ok(v) => v,
                    Err(()) => return Err(()),
                };
                let right_val = match self.evaluate(right) {
                    Ok(v) => v,
                    Err(()) => return Err(()),
                };

                match operator.token_type {
                    TokenType::AND => Ok(Literals::Boolean(is_truthy(&left_val) && is_truthy(&right_val))),
                    TokenType::OR => Ok(Literals::Boolean(is_truthy(&left_val) || is_truthy(&right_val))),
                    TokenType::GREATER => {
                        match self.check_number_operand(operator, &left_val, &right_val) {
                            Ok(_) => Ok(Literals::Boolean(left_val.unwrap_number() > right_val.unwrap_number())),
                            Err(_) => Err(())
                        }
                    },
                    TokenType::GREATER_EQUAL => {
                        match self.check_number_operand(operator, &left_val, &right_val) {
                            Ok(_) => Ok(Literals::Boolean(left_val.unwrap_number() >= right_val.unwrap_number())),
                            Err(_) => Err(())
                        }
                    },
                    TokenType::LESS => {
                        match self.check_number_operand(operator, &left_val, &right_val) {
                            Ok(_) => Ok(Literals::Boolean(left_val.unwrap_number() < right_val.unwrap_number())),
                            Err(_) => Err(())
                        }
                    },
                    TokenType::LESS_EQUAL => {
                        match self.check_number_operand(operator, &left_val, &right_val) {
                            Ok(_) => Ok(Literals::Boolean(left_val.unwrap_number() <= right_val.unwrap_number())),
                            Err(_) => Err(())
                        }
                    },
                    TokenType::BANG_EQUAL => Ok(Literals::Boolean(!is_equal(&left_val, &right_val))),
                    TokenType::EQUAL_EQUAL => Ok(Literals::Boolean(is_equal(&left_val, &right_val))),
                    TokenType::MINUS => {
                        match self.check_number_operand(operator, &left_val, &right_val) {
                            Ok(_) => Ok(Literals::Number(left_val.unwrap_number() - right_val.unwrap_number())),
                            Err(_) => Err(())
                        }
                    },
                    TokenType::PLUS => {
                        match left_val {
                            Literals::Number(l) => { match right_val {
                                Literals::Number(r) => Ok(Literals::Number(l + r)),
                                _ => {
                                    self.report_err(operator.clone(),
                                                    format!("Operands of '{}' must be two numbers or two strings.", operator.lexeme));
                                    Err(())
                                }
                            }},
                            Literals::String(l) => { match right_val {
                                Literals::String(r) => Ok(Literals::String(format!("{}{}", l, r))),
                                _ => {
                                    self.report_err(operator.clone(),
                                                    format!("Operands of '{}' must be two numbers or two strings.", operator.lexeme));
                                    Err(())
                                }
                            }},
                            _ => {
                                self.report_err(operator.clone(),
                                                format!("Operands of '{}' must be two numbers or two strings.", operator.lexeme));
                                Err(())
                            },
                        }
                    },
                    TokenType::SLASH => {
                        match self.check_number_operand(operator, &left_val, &right_val) {
                            Ok(_) => Ok(Literals::Number(left_val.unwrap_number() / right_val.unwrap_number())),
                            Err(_) => Err(())
                        }
                    },
                    TokenType::STAR => {
                        match left_val {
                            Literals::Number(l) => { match right_val {
                                Literals::Number(r) => Ok(Literals::Number(l * r)),
                                Literals::String(r) => Ok(Literals::String(r.repeat(l as usize))),
                                _ => {
                                    self.report_err(operator.clone(),
                                                    format!("Operands of '{}' must be two numbers or a string and a number.", operator.lexeme));
                                    Err(())
                                }
                            }},
                            Literals::String(l) => { match right_val {
                                Literals::Number(r) => Ok(Literals::String(l.repeat(r as usize))),
                                _ => {
                                    self.report_err(operator.clone(),
                                                    format!("Operands of '{}' must be two numbers or a string and a number.", operator.lexeme));
                                    Err(())
                                }
                            }},
                            _ => {
                                self.report_err(operator.clone(),
                                                format!("Operands of '{}' must be two numbers or a string and a number.", operator.lexeme));
                                Err(())
                            }
                        }
                    },
                    _ => {
                        self.report_err(operator.clone(),
                                        format!("Unsupported operator: '{}'.", operator.lexeme));
                        Err(())
                    }
                }
            },

            // TODO: Implement visit Call expression.
            Expr::Call(callee, paren, arguments) => {
                let callee_val = self.evaluate(callee);

                let mut argument_vals = Vec::new();
                for argument in arguments.iter() {
                    argument_vals.push(self.evaluate(argument));
                }

                // temp code
                Ok(Literals::Nil)
            },

            Expr::Grouping(expression) => {
                self.evaluate(expression)
            },

            // TODO: Implement visit Get expression.
            Expr::Get(object, name) => {
                Ok(Literals::Nil)
            }

            Expr::IfExpr(condition, then_branch, else_branch) => {
                let condition_val = is_truthy(&self.evaluate(condition).unwrap());
                if condition_val {
                    self.execute(then_branch)
                } else {
                    self.execute(else_branch)
                }

                // temp code
                Ok(Literals::Nil)
            },

            // TODO: Implement visit Index expression.

            Expr::IndexGet(value, index) => {
                Ok(Literals::Nil)
            }

            Expr::IndexSet(expr, index, value) => {
                Ok(Literals::Nil)
            }

            Expr::Literal(value) => {
                Ok(value.clone())
            },

            // TODO: Implement visit Set expression.
            Expr::Set(object, name, value) => {
                Ok(Literals::Nil)
            }

            // TODO: Implement visit Self expression.
            Expr::SelfExpr(keyword) => {
                Ok(Literals::Nil)
            }

            // TODO: Implement visit Super expression.
            Expr::SuperExpr(keyword, method) => {
                Ok(Literals::Nil)
            }

            Expr::Unary(operator, right) => {
                let right_val = self.evaluate(right).unwrap();

                match operator.token_type {
                    TokenType::BANG | TokenType::NOT => Ok(Literals::Boolean(!is_truthy(&right_val))),
                    TokenType::MINUS => { match right_val {
                        Literals::Number(n) => Ok(Literals::Number(-n)),
                        _ => {
                            self.report_err(operator.clone(), format!("Operand of '{}' must be a number.", operator.lexeme));
                            Err(())
                        }
                    }},
                    _ => {
                        self.report_err(operator.clone(), format!("Operand of '{}' must be a number.", operator.lexeme));
                        Err(())
                    }
                }
            },

            Expr::Variable(name) => {
                let res = self.environment.borrow().get(name);
                match res {
                    Ok(literal) => Ok(literal),
                    Err(_) => {
                        self.report_err(name.clone(), format!("Variable '{}' not found in scope.", name.lexeme));
                        Err(())
                    }
                }
            },
        }
    }
}


impl StmtVisitor for Interpreter {
    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(statements) => self.execute_block(statements, Environment::new(Some(self.environment.clone()))),

            // TODO: Implement visit Break statement.
            Stmt::Break => {},

            // TODO: Implement visit Continue statement.
            Stmt::Continue => {},

            // TODO: Implement visit Class statement.
            Stmt::Class(name, superclass, methods) => {},

            Stmt::Expression(expression) => {
                let res = self.evaluate(expression);
                match res {
                    Ok(_) => {},
                    // TODO: Handle possible runtime error after add tokens to Stmt::Expression.
                    Err(_) => {}
                }
            },

            // TODO: Finish visit For statement.
            Stmt::For(var_name, range_name, body) => {
                let sub_env = Environment::new(Some(self.environment.clone()));

                // match range_name.token_type {
                //     TokenType::IDENTIFIER => {
                //     },
                //     _ => {}
                // }
            },

            // TODO: Implement visit Function statement.
            Stmt::Function(name, params, body) => {},

            Stmt::Print(expression) => {
                match self.evaluate(expression) {
                    Ok(literal) => println!("{}", stringify(literal)),
                    Err(_) => {}
                }
            },

            // TODO: Implement visit Return statement.
            Stmt::Return(expression) => {},

            Stmt::Variable(name, initializer) => {
                let val = match initializer {
                    Some(i) => match self.evaluate(i) {
                        Ok(literal) => literal,
                        Err(()) => Literals::Nil,
                    },
                    None => Literals::Nil,
                };
                self.environment.borrow_mut().define(name.clone(), val)
            },

            Stmt::While(condition, body) => {
                while is_truthy(&self.evaluate(condition).unwrap()) {
                    self.execute(body)
                }
            }
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

fn stringify(literal: Literals) -> String {
    match literal {
        Literals::Nil => "nil".to_string(),

        // Remove the '.0' at the end of integer-valued floats.
        Literals::Number(n) => n.to_string(),
        Literals::String(s) => s,
        Literals::Boolean(b) => b.to_string(),
    }
}
