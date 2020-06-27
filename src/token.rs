use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::dove_callable::DoveCallable;
use crate::dove_class::{DoveClass, DoveInstance};
use crate::data_types::DoveObject;

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
    AND, BREAK, CLASS, CONTINUE, ELSE, FALSE, FUN, FOR, FROM, IMPORT, IN, IF, LAMBDA, LET, NIL, NOT, OR,
    PRINT, RETURN, SUPER, SELF, TRUE, WHILE,

    // End of file.
    EOF
}

#[derive(Clone)]
pub enum Literals {
    Array(Rc<RefCell<Vec<Literals>>>),
    Dictionary(Rc<RefCell<HashMap<DictKey, Literals>>>),
    String(String),
    Tuple(Box<Vec<Literals>>),
    Number(f64),
    Boolean(bool),
    Nil,
    Function(Rc<dyn DoveCallable>),
    Class(Rc<DoveClass>),
    Instance(Rc<RefCell<DoveInstance>>),
}

impl std::fmt::Debug for Literals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TODO maybe")
    }
}

impl Literals {
    pub fn to_string(&self) -> String {
        match self {
            Literals::Array(_) => "Array".to_string(),
            Literals::Dictionary(_) => "Dictionary".to_string(),
            Literals::String(_) => "String".to_string(),
            Literals::Tuple(_) => "Tuple".to_string(),
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
    pub fn unwrap_usize(self) -> Result<usize, ()> {
        match self.unwrap_number() {
            Ok(n) => {
                if n.fract() != 0.0 || n < 0.0 { return Err(()); }
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

    pub fn as_object(&self) -> Box<dyn DoveObject> {
        match self {
            Literals::String(string) => Box::new(string.clone()),
            Literals::Instance(instance) => Box::new(Rc::clone(instance)),
            Literals::Array(array) => Box::new(Rc::clone(array)),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub enum DictKey {
    StringKey(String),
    NumberKey(isize),
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
