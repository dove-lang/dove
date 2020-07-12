use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::ast::*;
use crate::token::*;
use crate::error_handler::*;
use crate::dove_callable::*;
use crate::dove_class::{DoveClass, DoveInstance};
use crate::environment::Environment;
use crate::constants::keywords;
use crate::dove_output::DoveOutput;

/// An enum indicating that execution was interrupted, for some reason.
#[derive(Debug, Clone)]
pub enum Interrupt {
    Break,
    Continue,
    Return(Literals),
    Error(RuntimeError),
}

// This automatically converts Err(RuntimeError) to Err(Interrupt::Error(RntimeError))
impl From<RuntimeError> for Interrupt {
    fn from(error: RuntimeError) -> Self {
        Interrupt::Error(error)
    }
}

type Result<T> = std::result::Result<T, Interrupt>;

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    pub error_handler: RuntimeErrorHandler,
    /// Depth of local variables, keyed by (line number, variable name)
    locals: HashMap<(usize, String), usize>,

    output: Rc<dyn DoveOutput>,
}

impl Interpreter {
    pub fn new(output: Rc<dyn DoveOutput>) -> Interpreter {
        let env = Rc::new(RefCell::new(Environment::new(Option::None)));
        Interpreter{
            globals: env.clone(),
            environment: env.clone(),
            error_handler: RuntimeErrorHandler::new(Rc::clone(&output)),
            locals: HashMap::new(),
            output,
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) {
        for stmt in stmts.iter() {
            // As this function should only be used by Dove struct,
            // no return value should be expected.
            self.execute(stmt).unwrap_or_else(|interrupt| match interrupt {
                Interrupt::Error(error) => self.error_handler.runtime_error(error),
                _ => self.output.error(format!("Unexpected interrupt: {:?}", interrupt)),
            });
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Literals> {
        self.visit_expr(expr)
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        self.visit_stmt(stmt)
    }

    pub fn execute_block(&mut self, statements: &Vec<Stmt>, environment: Environment) -> Result<()> {
        let previous = std::mem::replace(&mut self.environment, Rc::new(RefCell::new(environment)));

        for stmt in statements.iter() {
            match self.execute(stmt) {
                Ok(_) => {},
                Err(err) => {
                    self.environment = previous;
                    return Err(err);
                },
            }
        }

        self.environment = previous;
        Ok(())
    }

    /// Execute a block and return its implicit return value.
    pub fn execute_implicit_return(&mut self, statements: &Vec<Stmt>, environment: Environment) -> Result<Literals> {
        // Check if last statement is an expression
        if let Some(Stmt::Expression(expr)) = statements.last() {
            let previous = std::mem::replace(&mut self.environment, Rc::new(RefCell::new(environment)));

            // Iterate through all statements except the last
            for stmt in statements[..statements.len() - 1].iter() {
                match self.execute(stmt) {
                    Ok(_) => {},
                    Err(err) => {
                        self.environment = previous;
                        return Err(err);
                    },
                }
            }

            // Evaluate the last expression
            let return_value = match self.evaluate(expr) {
                Ok(value) => value,
                Err(err) => {
                    self.environment = previous;
                    return Err(err);
                }
            };

            self.environment = previous;
            Ok(return_value)

        } else {
            // No implicit return, execute as normal and return nil
            self.execute_block(statements, environment)?;
            Ok(Literals::Nil)
        }
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

    fn lookup_variable(&self, variable: &Token) -> Option<Literals> {
        match self.get_local(variable) {
            Some(distance) => self.environment.borrow().get_at(*distance, &variable.lexeme),
            None => self.globals.borrow().get(&variable.lexeme),
        }
    }

    fn check_number_operand(&mut self, operator: &Token, left: &Literals, right: &Literals) -> Result<(f64, f64)> {
        match (left, right) {
            (Literals::Number(l), Literals::Number(r)) => Ok((*l, *r)),
            _ => Err(Interrupt::Error(RuntimeError::new(
                ErrorLocation::Token(operator.clone()),
                format!("Operands of '{}' must be two numbers.", operator.lexeme),
            ))),
        }
    }

    fn check_integer_operand(&mut self, operator: &Token, left: &Literals, right: &Literals) -> Result<(i32, i32)> {
        match self.check_number_operand(operator, left, right) {
            Ok((l, r)) if l.fract() == 0.0 && r.fract() == 0.0 => Ok((l as i32, r as i32)),
            _ => Err(Interrupt::Error(RuntimeError::new(
                ErrorLocation::Token(operator.clone()),
                format!("Operands of '{}' must be two integers.", operator.lexeme)),
            )),
        }

    }

    // fn report_err(&mut self, token: Token, message: String) {
    //     let rt_err = RuntimeError::new(token.clone(), message);
    //     self.error_handler.runtime_error(rt_err);
    // }
}

impl ExprVisitor for Interpreter {
    type Result = Result<Literals>;

    fn visit_expr(&mut self, expr: &Expr) -> Result<Literals> {
        match expr {
            Expr::Array(expressions) => {
                let mut arr_vals = Vec::new();
                for expr in expressions {
                    arr_vals.push(self.evaluate(expr)?);
                }
                Ok(Literals::Array(Rc::new(RefCell::new(arr_vals))))
            },

            Expr::Assign(name, op, value) => {
                let line = op.line;
                let val = match op.token_type {
                    TokenType::EQUAL => {
                        self.evaluate(value)?
                    },
                    TokenType::PLUS_EQUAL | TokenType::PLUS_PLUS => {
                        self.evaluate(&Expr::Binary(Box::new(Expr::Variable(name.clone())),
                                                         Token::new(TokenType::PLUS, "+".to_string(), None, line),
                                                         value.clone()))?
                    },
                    TokenType::MINUS_EQUAL | TokenType::MINUS_MINUS => {
                        self.evaluate(&Expr::Binary(Box::new(Expr::Variable(name.clone())),
                                                    Token::new(TokenType::MINUS, "-".to_string(), None, line),
                                                    value.clone()))?
                    },
                    TokenType::STAR_EQUAL => {
                        self.evaluate(&Expr::Binary(Box::new(Expr::Variable(name.clone())),
                                                    Token::new(TokenType::STAR, "*".to_string(), None, line),
                                                    value.clone()))?
                    },
                    TokenType::SLASH_EQUAL => {
                        self.evaluate(&Expr::Binary(Box::new(Expr::Variable(name.clone())),
                                                    Token::new(TokenType::SLASH, "/".to_string(), None, line),
                                                    value.clone()))?
                    }
                    _ => panic!("Magically found non assignment operator wrapped inside an Expr::Assign.")
                };

                let assigned = match self.get_local(name) {
                    Some(distance) => self.environment.borrow_mut().assign_at(*distance, name.lexeme.clone(), val.clone()),
                    None => self.globals.borrow_mut().assign(name.lexeme.clone(), val.clone()),
                };

                if assigned {
                    Ok(val)
                } else {
                    Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Token(name.clone()),
                        format!("Cannot assign value to '{}', as it is not found in scope.", name.lexeme),
                    )))
                }
            },

            Expr::Binary(left, operator, right) => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::AND => Ok(Literals::Boolean(is_truthy(&left_val) && is_truthy(&right_val))),
                    TokenType::OR => Ok(Literals::Boolean(is_truthy(&left_val) || is_truthy(&right_val))),
                    TokenType::GREATER => {
                        let (left_val, right_val) = self.check_number_operand(operator, &left_val, &right_val)?;
                        Ok(Literals::Boolean(left_val > right_val))
                    },
                    TokenType::GREATER_EQUAL => {
                        let (left_val, right_val) = self.check_number_operand(operator, &left_val, &right_val)?;
                        Ok(Literals::Boolean(left_val >= right_val))
                    },
                    TokenType::LESS => {
                        let (left_val, right_val) = self.check_number_operand(operator, &left_val, &right_val)?;
                        Ok(Literals::Boolean(left_val < right_val))
                    },
                    TokenType::LESS_EQUAL => {
                        let (left_val, right_val) = self.check_number_operand(operator, &left_val, &right_val)?;
                        Ok(Literals::Boolean(left_val <= right_val))
                    },
                    TokenType::BANG_EQUAL => Ok(Literals::Boolean(!is_equal(&left_val, &right_val))),
                    TokenType::EQUAL_EQUAL => Ok(Literals::Boolean(is_equal(&left_val, &right_val))),
                    TokenType::MINUS => {
                        let (left_val, right_val) = self.check_number_operand(operator, &left_val, &right_val)?;
                        Ok(Literals::Number(left_val - right_val))
                    },
                    TokenType::PERCENT => {
                        let (left_val, right_val) = self.check_number_operand(operator, &left_val, &right_val)?;
                        Ok(Literals::Number(left_val % right_val))
                    }
                    TokenType::PLUS => {
                        match (left_val, right_val) {
                            (Literals::Number(l), Literals::Number(r)) => Ok(Literals::Number(l + r)),
                            (Literals::String(l), Literals::String(r)) => Ok(Literals::String(format!("{}{}", l, r))),
                            (Literals::String(l), Literals::Number(r)) => Ok(Literals::String(format!("{}{}", l, r))),
                            (Literals::Number(l), Literals::String(r)) => Ok(Literals::String(format!("{}{}", l, r))),
                            (Literals::Array(l), Literals::Array(r)) => {
                                let mut res = Vec::new();
                                for val in l.borrow().iter() {
                                    res.push(val.clone());
                                }
                                for val in r.borrow().iter() {
                                    res.push(val.clone());
                                }

                                Ok(Literals::Array(Rc::new(RefCell::new(res))))
                            }
                            (Literals::Tuple(l), Literals::Tuple(r)) => {
                                let mut res = Vec::new();
                                for val in *l {
                                    res.push(val);
                                }
                                for val in *r {
                                    res.push(val);
                                }

                                Ok(Literals::Tuple(Box::new(res)))
                            }
                            _ => Err(Interrupt::Error(RuntimeError::new(
                                ErrorLocation::Token(operator.clone()),
                                format!("Operands of '{}' must be two numbers/strings/arrays/tuples.", operator.lexeme),
                            ))),
                        }
                    },
                    TokenType::SLASH => {
                        let (left_val, right_val) = self.check_number_operand(operator, &left_val, &right_val)?;
                        Ok(Literals::Number(left_val / right_val))
                    },
                    TokenType::SLASH_GREATER => {
                        let (left_val, right_val) = self.check_number_operand(operator, &left_val, &right_val)?;
                        Ok(Literals::Number((left_val / right_val).ceil()))
                    },
                    TokenType::SLASH_LESS => {
                        let (left_val, right_val) = self.check_number_operand(operator, &left_val, &right_val)?;
                        Ok(Literals::Number((left_val / right_val).floor()))
                    },
                    TokenType::STAR => {
                        match (left_val, right_val) {
                            (Literals::Number(l), Literals::Number(r)) => Ok(Literals::Number(l * r)),
                            (Literals::Number(l), Literals::String(r)) => Ok(Literals::String(r.repeat(l as usize))),
                            (Literals::String(l), Literals::Number(r)) => Ok(Literals::String(l.repeat(r as usize))),
                            _ => Err(Interrupt::Error(RuntimeError::new(
                                ErrorLocation::Token(operator.clone()),
                                format!("Operands of '{}' must be two numbers or a string and a number.", operator.lexeme),
                            ))),
                        }
                    },
                    TokenType::DOT_DOT => {
                        let (left, right) = self.check_integer_operand(operator, &left_val, &right_val)?;

                        let is_right_bigger = right >= left;
                        let diff = (right - left).abs();

                        let mut res = Vec::new();
                        for i in 0..diff {
                            let new = left + i * if is_right_bigger { 1 } else { -1 };
                            res.push(Literals::Number(new as f64));
                        }

                        Ok(Literals::Tuple(Box::new(res)))
                    },
                    TokenType::DOT_DOT_DOT => {
                        let (left, right) = self.check_integer_operand(operator, &left_val, &right_val)?;

                        let is_right_bigger = right >= left;
                        let diff = (right - left).abs();

                        let mut res = Vec::new();
                        for i in 0..(diff + 1) {
                            let new = left + i * if is_right_bigger { 1 } else { -1 };
                            res.push(Literals::Number(new as f64));
                        }

                        Ok(Literals::Tuple(Box::new(res)))
                    },
                    _ =>Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Token(operator.clone()),
                        format!("Unsupported operator: '{}'.", operator.lexeme)),
                    )),
                }
            },

            Expr::Call(callee, paren, arguments) => {
                let callee_val = self.evaluate(callee)?;
                let callee_type = (&callee_val).to_string();

                // Evaluate argument literals.
                let mut argument_vals = Vec::new();
                for argument in arguments.iter() {
                    argument_vals.push(self.evaluate(argument)?);
                }

                // TODO: simplify
                match callee_val {
                    Literals::Class(class) => {
                        let instance = Rc::new(RefCell::new(DoveInstance::new(Rc::clone(&class))));

                        if let Some(initializer) = class.find_method("init") {
                            let bound_init = initializer.bind(Rc::clone(&instance));

                            // TODO: move this somewhere else? inside function.call?
                            if argument_vals.len() != bound_init.arity() {
                                return Err(Interrupt::Error(RuntimeError::new(
                                    ErrorLocation::Token(paren.clone()),
                                    format!("Expected {} arguments but got {}.", bound_init.arity(), argument_vals.len()),
                                )));
                            }

                            bound_init.call(self, &argument_vals)?;
                        }

                        Ok(Literals::Instance(instance))
                    },
                    Literals::Function(function) => {
                        // Check arity.
                        if argument_vals.len() != function.arity() {
                            return Err(Interrupt::Error(RuntimeError::new(
                                ErrorLocation::Token(paren.clone()),
                                format!("Expected {} arguments but got {}.", function.arity(), argument_vals.len()),
                            )));
                        }

                        Ok(function.call(self, &argument_vals)?)
                    },
                    _ => Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Token(paren.clone()),
                        format!("Type '{}' is not callable.", callee_type),
                    ))),
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
                        Literals::Number(key) if key.fract() != 0.0 =>{
                            dict_val.insert(DictKey::NumberKey(key as isize), val);
                        },

                        _ => return Err(Interrupt::Error(RuntimeError::new(
                            ErrorLocation::Unspecified,
                            "Only String and Integer can be used as dictionary key.".to_string(),
                        ))),
                    };
                }
                Ok(Literals::Dictionary(Rc::new(RefCell::new(dict_val))))
            },

            Expr::Grouping(expression) => {
                self.evaluate(expression)
            },

            Expr::Get(object, name) => {
                let expr = self.visit_expr(object)?;

                match expr.as_object().get_property(&name.lexeme) {
                    Ok(value) => Ok(value),
                    Err(_) => Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Token(name.clone()),
                        format!("Cannot get property '{}' of type '{}'.", name.lexeme, expr.to_string()),
                    ))),
                }
            }

            Expr::IfExpr(condition, then_branch, else_branch) => {
                let condition_val = is_truthy(&self.evaluate(condition)?);

                let branch = if condition_val {
                    then_branch
                } else {
                    else_branch
                };

                let statements = match branch.as_ref() {
                    Stmt::Block(statements) => statements,
                    _ => panic!("If statement has a non-block branch"),
                };

                let env = Environment::new(Some(self.environment.clone()));
                let value = self.execute_implicit_return(statements, env)?;

                Ok(value)
            },

            Expr::IndexGet(expr, index) => {
                let evaluated_expr = self.evaluate(expr)?;
                let evaluated_index = self.evaluate(index)?;

                match evaluated_expr {
                    Literals::Array(arr) => {
                        match evaluated_index.unwrap_usize() {
                            Ok(n) => match arr.borrow().get(n) {
                                Some(v) => Ok(v.clone()),
                                None => Err(Interrupt::Error(RuntimeError::new(
                                    ErrorLocation::Unspecified,
                                    format!("Index '{}' out of range.", n),
                                ))),
                            },
                            Err(_) => Err(Interrupt::Error(RuntimeError::new(
                                ErrorLocation::Unspecified,
                                "Index must be an integer.".to_string(),
                            ))),
                        }
                    },
                    Literals::Tuple(tup) => {
                        match evaluated_index.unwrap_usize() {
                            Ok(n) => match tup.get(n) {
                                Some(v) => Ok(v.clone()),
                                None => Err(Interrupt::Error(RuntimeError::new(
                                    ErrorLocation::Unspecified,
                                    format!("Index '{}' out of range.", n),
                                ))),
                            },
                            Err(_) => Err(Interrupt::Error(RuntimeError::new(
                                ErrorLocation::Unspecified,
                                "Index must be an integer.".to_string(),
                            ))),
                        }
                    },
                    Literals::Dictionary(dict) => {
                        let dict_key = match evaluated_index {
                            Literals::Number(i) if i.fract() != 0.0 => DictKey::NumberKey(i as isize),
                            Literals::String(s) => DictKey::StringKey(s.clone()),
                            _ => return Err(Interrupt::Error(RuntimeError::new(
                                ErrorLocation::Unspecified,
                                "Index must be an integer/string.".to_string(),
                            ))),
                        };

                        match dict.borrow().get(&dict_key) {
                            Some(v) => Ok(v.clone()),
                            None => Err(Interrupt::Error(RuntimeError::new(
                                ErrorLocation::Unspecified,
                                format!("Key '{}' not found.", dict_key.stringify()),
                            )))
                        }
                    },
                    _ => Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Unspecified,
                        format!("Cannot get value by index/key from '{}'.", evaluated_expr.to_string()),
                    ))),
                }
            }

            Expr::IndexSet(expr, index, value) => {
                let evaluated_expr = self.evaluate(expr)?;
                let evaluated_index = self.evaluate(index)?;
                let evaluated_value = self.evaluate(value)?;

                match evaluated_expr {
                    Literals::Array(arr) => {
                        match evaluated_index.unwrap_usize() {
                            Ok(n) => {
                                let old_val = match arr.borrow().get(n) {
                                    Some(v) => v.clone(),
                                    None => return Err(Interrupt::Error(RuntimeError::new(
                                        ErrorLocation::Unspecified,
                                        format!("Index '{}' out of range.", n),
                                    ))),
                                };
                                // Index must exist, otherwise already returned Err(Interrupt::Error).
                                arr.borrow_mut()[n] = evaluated_value;
                                Ok(old_val)
                            },
                            Err(_) => Err(Interrupt::Error(RuntimeError::new(
                                ErrorLocation::Unspecified,
                                "Index must be an integer.".to_string(),
                            ))),
                        }
                    },
                    Literals::Dictionary(dict) => {
                        let dict_key = match evaluated_index {
                            Literals::Number(i) if i.fract() != 0.0 => DictKey::NumberKey(i as isize),
                            Literals::String(s) => DictKey::StringKey(s.clone()),
                            _ => return Err(Interrupt::Error(RuntimeError::new(
                                ErrorLocation::Unspecified,
                                "Index must be an integer/string.".to_string(),
                            ))),
                        };

                        let old_val = match dict.borrow().get(&dict_key) {
                            Some(v) => v.clone(),
                            None => Literals::Nil,
                        };
                        dict.borrow_mut().insert(dict_key, evaluated_value);
                        Ok(old_val)
                    }
                    _ => Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Unspecified,
                        format!("Cannot set value by index/key from '{}'.", evaluated_expr.to_string()),
                    ))),
                }
            }

            Expr::Lambda(params, body) => {
                let lambda = DoveFunction::new(params.clone(), *body.clone(), Rc::clone(&self.environment));
                Ok(Literals::Function(Rc::new(lambda)))
            }

            Expr::Literal(value) => {
                Ok(value.clone())
            },

            Expr::Set(object, name, value) => {
                let expr = self.visit_expr(object)?;
                let value = self.visit_expr(value)?;

                match expr.as_object().set_property(&name.lexeme, value.clone()) {
                    Ok(_) => Ok(value),
                    Err(_) => Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Token(name.clone()),
                        format!("Cannot get property '{}' of type '{}'.", name.lexeme, expr.to_string()),
                    )))
                }
            },

            Expr::SelfExpr(token) => {
                if let Some(instance) = self.lookup_variable(token) {
                    Ok(instance)
                } else {
                    Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Token(token.clone()),
                        format!("Cannot find '{}' in the scope.", token.lexeme),
                    )))
                }
            }

            Expr::SuperExpr(token, method) => {
                // Get distance to super to be used for self later
                let distance = match self.get_local(token) {
                    Some(distance) => *distance,
                    None => return Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Token(token.clone()),
                        format!("Cannot resolve '{}' in the scope.", token.lexeme),
                    ))),
                };

                let maybe_class = self.environment.borrow().get_at(distance, &token.lexeme);
                let class = match maybe_class {
                    Some(Literals::Class(class)) => class,
                    _ => return Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Token(token.clone()),
                        "Cannot find superclass.".to_string(),
                    ))),
                };

                let method = match class.find_method(&method.lexeme) {
                    Some(method) => method,
                    None => return Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Token(method.clone()),
                        format!("Cannot find method '{}' from class '{}'", method.lexeme, class.name),
                    ))),
                };

                // TODO: find a more "elegant" solution. If so, remember to change visit super/self in resolver
                // TODO: consider for static methods?
                let maybe_instance = self.environment.borrow().get_at(distance - 1, &keywords::SELF);
                let instance = match maybe_instance {
                    Some(Literals::Instance(instance)) => instance,
                    _ => return Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Line(token.line),
                        "Cannot find 'self' in the scope".to_string(),
                    ))),
                };

                let bound_method = method.bind(instance);
                Ok(Literals::Function(Rc::new(bound_method)))
            }

            Expr::Tuple(expressions) => {
                let mut tup_vals = Vec::new();
                for expr in expressions {
                    tup_vals.push(self.evaluate(expr)?);
                }
                Ok(Literals::Tuple(Box::new(tup_vals)))
            },

            Expr::Unary(operator, right) => {
                let right_val = self.evaluate(right).unwrap();

                match operator.token_type {
                    TokenType::BANG | TokenType::NOT => Ok(Literals::Boolean(!is_truthy(&right_val))),
                    TokenType::MINUS => match right_val {
                        Literals::Number(n) => Ok(Literals::Number(-n)),
                        _ => Err(Interrupt::Error(RuntimeError::new(
                            ErrorLocation::Token(operator.clone()),
                            format!("Operand of '{}' must be a number.", operator.lexeme),
                        ))),
                    },
                    _ => Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Token(operator.clone()),
                        format!("Unsupported unary operator {}.", operator.lexeme),
                    ))),
                }
            },

            Expr::Variable(name) => {
                if let Some(value) = self.lookup_variable(name) {
                    Ok(value)
                } else {
                    Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Token(name.clone()),
                        format!("Variable '{}' not found in scope.", name.lexeme)),
                    ))
                }
            },
        }
    }
}


impl StmtVisitor for Interpreter {
    type Result = Result<()>;

    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Block(statements) => {
                self.execute_block(statements, Environment::new(Some(self.environment.clone())))
            },

            Stmt::Break(_) => {
                Err(Interrupt::Break)
            },

            Stmt::Continue(_) => {
                Err(Interrupt::Continue)
            },

            Stmt::Class(name, superclass_name, methods) => {
                let mut methods_map = HashMap::new();

                let mut superclass = None;

                if let Some(superclass_name) = superclass_name {
                    if let Some(Literals::Class(class)) = self.lookup_variable(superclass_name) {
                        superclass = Some(class);
                    } else {
                        return Err(Interrupt::Error(RuntimeError::new(
                            ErrorLocation::Token(superclass_name.clone()),
                            format!("Cannot find the class named '{}'.", superclass_name.lexeme),
                        )));
                    }
                }

                for method in methods {
                    let mut environment = Rc::clone(&self.environment);

                    let (name, params, body) = match method {
                        Stmt::Function(name, params, body) => (name, params, body),
                        _ => panic!("Class contains non-method statements."),
                    };

                    if let Some(superclass) = &superclass {
                        environment = Rc::new(RefCell::new(Environment::new(Some(environment))));
                        environment.borrow_mut().define(
                            keywords::SUPER.to_string(),
                            Literals::Class(Rc::clone(superclass)),
                        );
                    }

                    let function = Rc::new(DoveFunction::new(params.clone(), *body.clone(), environment));
                    methods_map.insert(name.lexeme.clone(), function);
                }

                let class = Rc::new(DoveClass::new(name.lexeme.clone(), superclass, methods_map));

                self.environment.borrow_mut().define(name.lexeme.clone(), Literals::Class(class));

                Ok(())
            },

            Stmt::Expression(expression) => {
                self.evaluate(expression)?;
                Ok(())
            },

            Stmt::For(var_name, range_name, body) => {
                let range_vals = self.evaluate(range_name)?;
                let stmts = match &**body {
                    Stmt::Block(stmts) => stmts,
                    _ => return Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Token(var_name.clone()),
                        "Expected block statement in a 'for' loop.".to_string(),
                    ))),
                };

                match range_vals {
                    Literals::Array(arr) => {

                        // Use loop with index to avoid having a reference to arr while executing `stmts`
                        let mut index = 0;

                        loop {
                            let item = match arr.borrow().get(index) {
                                Some(item) => item.clone(),
                                None => break,
                            };
                            // Reference to arr is dropped here

                            let mut sub_env = Environment::new(Some(self.environment.clone()));
                            sub_env.define(var_name.lexeme.clone(), item);

                            if let Err(interrupt) = self.execute_block(&stmts, sub_env) {
                                match interrupt {
                                    Interrupt::Break => return Ok(()),
                                    Interrupt::Continue => {},
                                    _ => return Err(interrupt),
                                }
                            }

                            index += 1;
                        }

                        Ok(())
                    },

                    Literals::Tuple(t) => {
                        let tup = *t;

                        for item in tup.iter() {
                            let mut sub_env = Environment::new(Some(self.environment.clone()));
                            sub_env.define(var_name.lexeme.clone(), item.clone());

                            if let Err(interrupt) = self.execute_block(&stmts, sub_env) {
                                match interrupt {
                                    Interrupt::Break => return Ok(()),
                                    Interrupt::Continue => {},
                                    _ => return Err(interrupt),
                                }
                            }
                        }

                        Ok(())
                    }

                    _ => Err(Interrupt::Error(RuntimeError::new(
                        ErrorLocation::Token(var_name.clone()),
                        format!("Cannot iterate over type '{}'", range_vals.to_string())
                    ))),
                }
            },

            Stmt::Function(name, params, body) => {
                // Convert DoveFunction to Function Literal.
                let function = DoveFunction::new(params.clone(), *body.clone(), Rc::clone(&self.environment));
                let function_literal = Literals::Function(Rc::new(function));
                self.environment.borrow_mut().define(name.lexeme.clone(), function_literal);
                Ok(())
            },

            Stmt::Print(_, expression) => {
                let literal = self.evaluate(expression)?;
                self.output.print(stringify(literal));
                Ok(())
            },

            Stmt::Return(_, expression) => {
                let value = match expression {
                    Some(expression) => self.evaluate(expression)?,
                    None => Literals::Nil,
                };
                Err(Interrupt::Return(value))
            },

            Stmt::Variable(name, initializer) => {
                let val = match initializer {
                    Some(i) => self.evaluate(i)?,
                    None => Literals::Nil,
                };
                self.environment.borrow_mut().define(name.lexeme.clone(), val);
                Ok(())
            },

            Stmt::While(condition, body) => {
                while is_truthy(&self.evaluate(condition).unwrap()) {
                     match self.execute(body) {
                         Ok(_) => {},
                         Err(interrupt) => {
                             match interrupt {
                                 Interrupt::Break => { return Ok(()); },
                                 Interrupt::Continue => { continue; }
                                 _ => { return Err(interrupt); }
                             }
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
                return if a.borrow().len() != other.borrow().len() {
                    false
                } else {
                    for i in 0..a.borrow().len() {
                        if !is_equal(&a.borrow()[i], &other.borrow()[i]) { return false; }
                    }
                    true
                };
            },
            _ => false,
        }},
        Literals::Dictionary(d) => { match literal_b {
            Literals::Dictionary(other) => {
                return if d.borrow().len() != other.borrow().len() {
                    false
                } else {
                    for (key, val) in d.borrow().iter() {
                        let mut flag = true;
                        match other.borrow().get(key) {
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
        _ => panic!("Comparison not supported.")
    }
}

fn stringify(literal: Literals) -> String {
    match literal {
        Literals::Array(a) => {
            let mut res = String::from("[");
            let arr = a.borrow();
            for item in arr.iter() {
                res.push_str(&format!("{}, ", stringify(item.clone())));
            }
            if res.len() > 1 {
                res.truncate(res.len() - 2);
            }
            res.push(']');
            res
        },
        Literals::Dictionary(h) => {
            let mut res = String::from("{");
            for (key, val) in h.borrow().iter() {
                res.push_str(&format!("{}: {}, ", key.stringify(), stringify(val.clone())));
            }
            if res.len() > 1 {
                res.truncate(res.len() - 2);
            }
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
            if res.len() > 1 {
                res.truncate(res.len() - 2);
            }
            res.push(')');
            res
        },
        Literals::Number(n) => n.to_string(),
        Literals::Boolean(b) => b.to_string(),
        Literals::Nil => "nil".to_string(),
        Literals::Function(_function) => {
            let mut res = String::from("<fun (");
            // TODO: hmm
            // for param in function.params.iter() {
            //     res.push_str(&param.lexeme);
            //     res.push_str(", ");
            // }
            if res.len() > 9 { res.truncate(res.len() - 2); }
            res.push_str(")>");

            res
        },
        _ => panic!("Not implemented.")
    }
}
