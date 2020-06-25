use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::{Stmt, Expr};
use crate::dove_callable::{DoveFunction, DoveLambda};
use crate::environment::Environment;
use crate::dove_class::{DoveClass, DoveInstance};

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literals>,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Option<Literals>, line: usize) -> Token {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl Token {
    pub fn to_string(&self) -> String {
        format!("type: {:?}, lexeme: {:?}, literal: {:?}, line: {}", self.token_type, self.lexeme, self.literal, self.line)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN, RIGHT_PAREN, LEFT_BRACE, RIGHT_BRACE, LEFT_BRACKET, RIGHT_BRACKET,
    COMMA, COLON, NEWLINE, PERCENT,

    // One or two character tokens.
    SLASH, SLASH_EQUAL, SLASH_LESS, SLASH_GREATER,
    STAR, STAR_EQUAL,
    BACKSLASH,
    BANG, BANG_EQUAL,
    EQUAL, EQUAL_EQUAL,
    GREATER, GREATER_EQUAL,
    PLUS, PLUS_EQUAL, PLUS_PLUS,
    MINUS, MINUS_EQUAL, MINUS_GREATER, MINUS_MINUS,
    LESS, LESS_EQUAL,

    // One or two or three character tokens.
    DOT, DOT_DOT, DOT_DOT_DOT,

    // Literals.
    IDENTIFIER, STRING, NUMBER,

    // Keywords.
    AND, ARRAY, BREAK, CLASS, CONTINUE, DICT, ELSE, FALSE, FUN, FOR, FROM, IN, IF, LAMBDA, LET, NIL, NOT, OR,
    PRINT, RETURN, SUPER, SELF, TRUE, TUPLE, WHILE,

    // End of file.
    EOF
}

#[derive(Debug, Clone)]
pub enum Literals {
    Array(Rc<RefCell<Vec<Literals>>>),
    Break,
    Continue,
    Dictionary(Rc<RefCell<HashMap<DictKey, Literals>>>),
    String(String),
    Tuple(Box<Vec<Literals>>),
    Lambda(Rc<DoveLambda>),
    Number(f64),
    Boolean(bool),
    Nil,
    Function(Rc<DoveFunction>),
    Class(Rc<DoveClass>),
    Instance(Rc<RefCell<DoveInstance>>),
}

impl Literals {
    pub fn to_string(&self) -> String {
        match self {
            Literals::Array(_) => "Array".to_string(),
            Literals::Break => "Break".to_string(),
            Literals::Continue => "Continue".to_string(),
            Literals::Dictionary(_) => "Dictionary".to_string(),
            Literals::String(_) => "String".to_string(),
            Literals::Tuple(_) => "Tuple".to_string(),
            Literals::Lambda(_) => "Lambda".to_string(),
            Literals::Number(_) => "Number".to_string(),
            Literals::Boolean(_) => "Boolean".to_string(),
            Literals::Nil => "Nil".to_string(),
            Literals::Function(_) => "Function".to_string(),
            Literals::Class(_) => "Class".to_string(),
            Literals::Instance(_) => "Instance".to_string(),
        }
    }

    pub fn unwrap_string(self) -> Result<String, ()> {
        match self {
            Literals::String(s) => Ok(s),
            _ => Err(())
        }
    }
    pub fn unwrap_number(self) -> Result<f64, ()> {
        match self {
            Literals::Number(n) => Ok(n),
            _ =>Err(())
        }
    }
    pub fn unwrap_int(self) -> Result<usize, ()> {
        match self.unwrap_number() {
            Ok(n) => {
                if n.fract() != 0.0 { return Err(()); }
                return Ok(n as usize);
            },
            Err(_) => Err(())
        }
    }
    pub fn unwrap_boolean(self) -> Result<bool, ()> {
        match self {
            Literals::Boolean(b) => Ok(b),
            _ => Err(())
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub enum DictKey {
    StringKey(String),
    NumberKey(usize),
}

impl DictKey {
    pub fn stringify(&self) -> String {
        match self {
            DictKey::StringKey(s) => format!("\"{}\"", s),
            DictKey::NumberKey(n) => n.to_string(),
        }
    }
}

impl PartialEq for DictKey {
    fn eq(&self, other: &Self) -> bool {
        match self {
            DictKey::StringKey(s) => match other {
                DictKey::StringKey(other_s) => s == other_s,
                DictKey::NumberKey(_) => false,
            },
            DictKey::NumberKey(n) => match other {
                DictKey::StringKey(_) => false,
                DictKey::NumberKey(other_n) => n == other_n,
            }
        }
    }
}

impl Eq for DictKey {}
