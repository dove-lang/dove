use std::collections::HashMap;
use lazy_static::lazy_static;

use crate::token::TokenType;

// Generate constants for keywords
macro_rules! keywords {
    ( $( $value:expr => $name:ident, )* ) => {
        $(
            pub static $name: &'static str = $value;
        )*

        lazy_static! {
            /// Map keyword `String`s to their corresponding `TokenType`
            pub static ref KEYWORD_TOKENS: HashMap<String, TokenType> = {
                let mut m = HashMap::new();
                $(
                    m.insert($value.to_string(), TokenType::$name);
                )*
                m
            };
        }
    }
}

keywords! {
    "and"       => AND,
    "break"     => BREAK,
    "class"     => CLASS,
    "continue"  => CONTINUE,
    "else"      => ELSE,
    "false"     => FALSE,
    "fun"       => FUN,
    "for"       => FOR,
    "from"      => FROM,
    "in"        => IN,
    "if"        => IF,
    "lambda"    => LAMBDA,
    "let"       => LET,
    "nil"       => NIL,
    "not"       => NOT,
    "or"        => OR,
    "print"     => PRINT,
    "return"    => RETURN,
    "super"     => SUPER,
    "self"      => SELF,
    "true"      => TRUE,
    "while"     => WHILE,
}
