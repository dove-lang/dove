use std::rc::Rc;
use std::cell::RefCell;

use crate::ast::*;
use crate::environment::Environment;
use crate::token::*;
use crate::error_handler::*;
use crate::dove_callable::*;
use crate::ast::Expr::Literal;

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    pub error_handler: RuntimeErrorHandler,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let env = Rc::new(RefCell::new(Environment::new(Option::None)));
        Interpreter{
            globals: env.clone(),
            environment: env.clone(),
            error_handler: RuntimeErrorHandler::new(),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) {
        for stmt in stmts.iter() {
            // As this function should only be used by Dove struct,
            // no return value should be expected.
            self.execute(stmt).unwrap_or_else(|return_val| {
                e_red_ln!("Unexpected return value: {}", return_val.to_string());
            });
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Literals, ()> {
        self.visit_expr(expr)
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), Literals> {
        self.visit_stmt(stmt)
    }

    pub fn execute_block(&mut self, statements: &Vec<Stmt>, environment: Environment) -> Result<(), Literals> {
        let previous = std::mem::replace(&mut self.environment, Rc::new(RefCell::new(environment)));

        for stmt in statements.iter() {
            match self.execute(stmt) {
                Ok(_) => {},
                Err(return_val) => {
                    self.environment = previous;
                    return Err(return_val);
                }
            }
        }

        self.environment = previous;
        Ok(())
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
            Expr::Array(expressions) => {
                let vals = c![self.evaluate(expr).unwrap(), for expr in expressions];
                Ok(Literals::Array(Box::new(vals)))
            },

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

            Expr::Call(callee, paren, arguments) => {
                let callee_val = match self.evaluate(callee) {
                    Ok(v) => v,
                    Err(_) => { return Err(()); }
                };
                let callee_type = (&callee_val).to_string();

                // Evaluate argument literals.
                let mut argument_vals = Vec::new();
                for argument in arguments.iter() {
                    argument_vals.push(match self.evaluate(argument) {
                        Ok(v) => v,
                        Err(_) => { return Err(()); }
                    });
                }

                // Try to convert the evaluated callee literal to a DoveFunction object.
                let mut function = match callee_val.to_function_object(){
                    Ok(f) => f,
                    Err(()) => {
                        self.report_err(paren.clone(), format!("Type '{}' is not callable.", callee_type));
                        return Err(());
                    }
                };

                // Check arity.
                if argument_vals.len() != function.arity() {
                    self.report_err(paren.clone(), format!("Expected {} arguments but got {}",
                                                           function.arity(), argument_vals.len()));
                    return Err(());
                }

                Ok(function.call(self, &argument_vals))
            },

            // TODO: Implement visit Dictionary expression
            Expr::Dictionary(expressions) => {
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
                    self.execute(then_branch);
                } else {
                    self.execute(else_branch);
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

            // TODO: Implement visit Tuple expression
            Expr::Tuple(expressions) => {
                Ok(Literals::Nil)
            },

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
    type Result = Literals;

    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<(), Self::Result> {
        match stmt {
            Stmt::Block(statements) => {
                self.execute_block(statements, Environment::new(Some(self.environment.clone())))
            },

            // TODO: Implement visit Break statement.
            Stmt::Break(_) => {Ok(())},

            // TODO: Implement visit Continue statement.
            Stmt::Continue(_) => {Ok(())},

            // TODO: Implement visit Class statement.
            Stmt::Class(name, superclass, methods) => {Ok(())},

            Stmt::Expression(expression) => {
                let res = self.evaluate(expression);
                match res {
                    Ok(_) => { Ok(()) },
                    // TODO: Handle possible runtime error after add tokens to Stmt::Expression.
                    Err(_) => { Ok(()) }
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
                Ok(())
            },

            Stmt::Function(name, params, body) => {
                // Convert DoveFunction to Function Literal.
                let function_literal = Literals::Function(Box::new(stmt.clone() ));
                self.environment.borrow_mut().define(name.clone(), function_literal);
                Ok(())
            },

            Stmt::Print(expression) => {
                match self.evaluate(expression) {
                    Ok(literal) => {
                        println!("{}", stringify(literal));
                        Ok(())
                    },
                    Err(_) => { Ok(()) }
                }
            },

            // TODO: Implement visit Return statement.
            Stmt::Return(expression) => {
                let value = match expression {
                    Some(expression) => self.evaluate(expression).unwrap(),
                    None => Literals::Nil,
                };
                Err(value)
            },

            Stmt::Variable(name, initializer) => {
                let val = match initializer {
                    Some(i) => match self.evaluate(i) {
                        Ok(literal) => literal,
                        Err(()) => Literals::Nil,
                    },
                    None => Literals::Nil,
                };
                self.environment.borrow_mut().define(name.clone(), val);
                Ok(())
            },

            Stmt::While(condition, body) => {
                while is_truthy(&self.evaluate(condition).unwrap()) {
                     match self.execute(body) {
                         Ok(_) => {},
                         Err(return_val) => {
                             return Err(return_val);
                         }
                     }
                }
                Ok(())
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
        _ => panic!("Not implemented.")
    }
}

fn stringify(literal: Literals) -> String {
    match literal {
        Literals::Array(a) => {
            let mut res = String::from("[");
            let arr = *a;
            for item in arr.iter() {
                res.push_str(&format!("{}, ", stringify(item.clone())));
            }
            res.truncate(res.len() - 2);
            res.push(']');
            res
        },
        Literals::String(s) => format!("\"{}\"", s),
        Literals::Number(n) => n.to_string(),
        Literals::Boolean(b) => b.to_string(),
        Literals::Nil => "nil".to_string(),
        Literals::Function(decla) => {
            let func_name = match *decla {
                Stmt::Function(name_token, _, _) => name_token.lexeme,
                _ => { panic!("Magically found non-function decalation wrapped inside Literals::Function."); }
            };
            format!("<fun {}>", func_name)
        }
        _ => panic!("Not implemented.")
    }
}
