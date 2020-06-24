use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::ast::*;
use crate::token::*;
use crate::error_handler::*;
use crate::dove_callable::*;
use crate::dove_class::{DoveClass, DoveInstance};
use crate::environment::Environment;

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    pub error_handler: RuntimeErrorHandler,
    /// Depth of local variables, keyed by (line number, variable name)
    locals: HashMap<(usize, String), usize>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let env = Rc::new(RefCell::new(Environment::new(Option::None)));
        Interpreter{
            globals: env.clone(),
            environment: env.clone(),
            error_handler: RuntimeErrorHandler::new(),
            locals: HashMap::new(),
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

    pub fn resolve(&mut self, token: &Token, depth: usize) {
        self.insert_local(token, depth);
    }

    fn insert_local(&mut self, variable: &Token, depth: usize) {
        self.locals.insert((variable.line, variable.lexeme.clone()), depth);
    }

    fn get_local(&self, variable: &Token) -> Option<&usize> {
        self.locals.get(&(variable.line, variable.lexeme.clone()))
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
                let mut arr_vals = Vec::new();
                for expr in expressions {
                    arr_vals.push(match self.evaluate(expr) {
                        Ok(v) => v,
                        Err(_) => { break; }
                    });
                }
                Ok(Literals::Array(Box::new(arr_vals)))
            },

            Expr::Assign(name, value) => {
                let val = self.evaluate(value)?;

                let res = match self.get_local(name) {
                    Some(distance) => self.environment.borrow_mut().assign_at(*distance, name.clone(), val.clone()),
                    None => self.globals.borrow_mut().assign(name.clone(), val.clone()),
                };

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

                // TODO: simplify
                match callee_val {
                    Literals::Class(class) => {
                        let instance = DoveInstance::new(Rc::clone(&class));
                        Ok(Literals::Instance(Rc::new(RefCell::new(instance))))
                    },
                    callee_val => {
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
                }
            },

            Expr::Dictionary(expressions) => {
                let mut dict_val = HashMap::new();
                for (key_expr, val_expr) in expressions.iter() {
                    let key = self.evaluate(key_expr).unwrap();
                    let val = self.evaluate(val_expr).unwrap();

                    // Check if key expr evaluates to String or Number.
                    match key {
                        Literals::String(key) => {
                            dict_val.insert(DictKey::StringKey(key), val);
                        },
                        Literals::Number(key) =>{
                            // Check if integer.
                            if key.fract() != 0.0 {
                                e_red_ln!("Only String and Integer can be used as dictionary key.");
                                return Err(());
                            }
                            dict_val.insert(DictKey::NumberKey(key as usize), val);
                        },
                        _ => {
                            e_red_ln!("Only String and Integer can be used as dictionary key.");
                            return Err(());
                        }
                    };
                }
                Ok(Literals::Dictionary(Box::new(dict_val)))
            },

            Expr::Grouping(expression) => {
                self.evaluate(expression)
            },

            // TODO: Implement visit Get expression.
            Expr::Get(object, name) => {
                let expr = self.visit_expr(object)?;

                match expr {
                    Literals::Instance(instance) => {
                        match instance.borrow().get(&name.lexeme) {
                            Some(value) => Ok(value.clone()),
                            None => {
                                self.error_handler.report(
                                    name.line,
                                    "".to_string(),
                                    format!("Undefined property '{}'.", name.lexeme),
                                );
                                Err(())
                            },
                        }
                    },
                    _ => {
                        // TODO: add properties for non-instance values? (ex. array.length)
                        self.error_handler.report(
                            name.line,
                            "".to_string(),
                            format!("Cannot get property '{}' from a non-instance value.", name.lexeme),
                        );
                        Err(())
                    }
                }
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
                let expr = self.visit_expr(object)?;
                let value = self.visit_expr(value)?;

                match expr {
                    Literals::Instance(instance) => {
                        instance.borrow_mut().set(name.lexeme.clone(), value.clone());
                        Ok(value.clone())
                    },
                    _ => {
                        self.error_handler.report(
                            name.line,
                            "".to_string(),
                            format!("Cannot set property '{}' of a non-instance value.", name.lexeme),
                        );
                        Err(())
                    }
                }
            }

            // TODO: Implement visit Self expression.
            Expr::SelfExpr(keyword) => {
                Ok(Literals::Nil)
            }

            // TODO: Implement visit Super expression.
            Expr::SuperExpr(keyword, method) => {
                Ok(Literals::Nil)
            }

            Expr::Tuple(expressions) => {
                let mut tup_vals = Vec::new();
                for expr in expressions {
                    tup_vals.push(match self.evaluate(expr) {
                        Ok(v) => v,
                        Err(_) => { break; }
                    });
                }
                Ok(Literals::Tuple(Box::new(tup_vals)))
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
                let res = match self.get_local(name) {
                    Some(distance) => self.environment.borrow().get_at(*distance, name),
                    None => self.globals.borrow().get(name),
                };

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

            Stmt::Class(name, superclass, methods) => {
                let class = Rc::new(DoveClass::new(name.lexeme.clone()));

                // TODO: define methods, superclasses, etc.

                self.environment.borrow_mut().define(name.clone(), Literals::Class(class));

                Ok(())
            },

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
                let function_literal = Literals::Function(Box::new(stmt.clone()), self.environment.clone());
                self.environment.borrow_mut().define(name.clone(), function_literal);
                Ok(())
            },

            Stmt::Print(_, expression) => {
                match self.evaluate(expression) {
                    Ok(literal) => {
                        println!("{}", stringify(literal));
                        Ok(())
                    },
                    Err(_) => { Ok(()) }
                }
            },

            // TODO: Implement visit Return statement.
            Stmt::Return(_, expression) => {
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
        Literals::Array(a) => { match literal_b {
            Literals::Array(other) => {
                return if a.len() != other.len() {
                    false
                } else {
                    for i in 0..a.len() {
                        if !is_equal(&a[i], &other[i]) { return false; }
                    }
                    true
                };
            },
            _ => false,
        }},
        Literals::Dictionary(d) => { match literal_b {
            Literals::Dictionary(other) => {
                return if d.len() != other.len() {
                    false
                } else {
                    for (key, val) in d.iter() {
                        let mut flag = true;
                        match other.get(key) {
                            Some(v) => if !is_equal(val, v) { flag = false; },
                            None => { flag = false; }
                        }
                        if !flag { return false; }
                    }
                    true
                };
            },
            _ => false,
        }},
        Literals::String(s) => { match literal_b {
            Literals::String(other) => s == other,
            _ => false,
        }},
        Literals::Tuple(t) => { match literal_b {
            Literals::Tuple(other) => {
                return if t.len() != other.len() {
                    false
                } else {
                    for i in 0..t.len() {
                        if !is_equal(&t[i], &other[i]) { return false; }
                    }
                    true
                };
            },
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
        Literals::Dictionary(h) => {
            let mut res = String::from("{");
            for (key, val) in *h {
                res.push_str(&format!("{}: {}, ", key.stringify(), stringify(val)));
            }
            res.truncate(res.len() - 2);
            res.push('}');
            res
        }
        Literals::String(s) => format!("\"{}\"", s),
        Literals::Tuple(a) => {
            let mut res = String::from("(");
            let arr = *a;
            for item in arr.iter() {
                res.push_str(&format!("{}, ", stringify(item.clone())));
            }
            res.truncate(res.len() - 2);
            res.push(')');
            res
        },
        Literals::Number(n) => n.to_string(),
        Literals::Boolean(b) => b.to_string(),
        Literals::Nil => "nil".to_string(),
        Literals::Function(decla, _) => {
            let func_name = match *decla {
                Stmt::Function(name_token, _, _) => name_token.lexeme,
                _ => { panic!("Magically found non-function decalation wrapped inside Literals::Function."); }
            };
            format!("<fun {}>", func_name)
        }
        _ => panic!("Not implemented.")
    }
}
