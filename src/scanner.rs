use crate::token::*;

pub struct Scanner {
    source: String,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner{ source }
    }
}

impl Scanner {
    pub fn scan_tokens(&self) -> Vec<Token> {
        vec![Token::new(TokenType::STRING, "test".to_string(), Literals::STRING("test".to_string()), 1)]
    }
}
