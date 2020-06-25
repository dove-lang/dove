use std::collections::HashMap;
use lazy_static::lazy_static;

use crate::token::*;
use crate::error_handler::*;

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,

    error_handler: CompiletimeErrorHandler,
}

impl Scanner {
    pub fn new(source: Vec<char>) -> Scanner {
        Scanner{
            source,
            tokens: Vec::new(),
            start: 0, current: 0, line: 1,
            error_handler: CompiletimeErrorHandler::new(),
        }
    }
}

impl Scanner {
    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() && !self.error_handler.had_error {
            // At the beginning of the next lexeme.
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            TokenType::EOF,
            "".to_string(),
            None,
            self.line
        ));

        &self.tokens
    }

    fn scan_token(&mut self) {
        let c: char = self.advance();
        match c {
            // One character.
            '(' => { self.add_token(TokenType::LEFT_PAREN, None); }
            ')' => { self.add_token(TokenType::RIGHT_PAREN, None); }
            '[' => { self.add_token(TokenType::LEFT_BRACKET, None); }
            ']' => { self.add_token(TokenType::RIGHT_BRACKET, None); }
            '{' => { self.add_token(TokenType::LEFT_BRACE, None); }
            '}' => { self.add_token(TokenType::RIGHT_BRACE, None); }
            ',' => { self.add_token(TokenType::COMMA, None); }
            ':' => { self.add_token(TokenType::COLON, None); }
            '%' => { self.add_token(TokenType::PERCENT, None); }
            // May be one or two characters.
            '+' => {
                if self.match_char('=') {
                    self.add_token(TokenType::PLUS_EQUAL, None);
                } else if self.match_char('+') {
                    self.add_token(TokenType::PLUS_PLUS, None);
                } else {
                    self.add_token(TokenType::PLUS, None);
                }
            }
            '*' => {
                let token_type = if self.match_char('=') { TokenType::STAR_EQUAL } else { TokenType::STAR };
                self.add_token(token_type, None);
            }
            '!' => {
                let token_type = if self.match_char('=') { TokenType::BANG_EQUAL } else { TokenType::BANG };
                self.add_token(token_type, None);
            }
            '=' => {
                let token_type = if self.match_char('=') { TokenType::EQUAL_EQUAL } else { TokenType::EQUAL };
                self.add_token(token_type, None);
            }
            '<' => {
                let token_type = if self.match_char('=') { TokenType::LESS_EQUAL } else { TokenType::LESS };
                self.add_token(token_type, None);
            }
            '>' => {
                let token_type = if self.match_char('=') { TokenType::GREATER_EQUAL } else { TokenType::GREATER };
                self.add_token(token_type, None);
            }
            // May be one or two or three characters.
            '.' => {
                let token_type = if self.match_char('.') {
                    if self.match_char('.') { TokenType::DOT_DOT_DOT } else { TokenType::DOT_DOT }
                } else { TokenType::DOT };
                self.add_token(token_type, None);
            }
            '-' => {
                if self.match_char('=') {
                    self.add_token(TokenType::MINUS_EQUAL, None);
                } else if self.match_char('-') {
                    self.add_token(TokenType::MINUS_MINUS, None);
                } else if self.match_char('>') {
                    self.add_token(TokenType::MINUS_GREATER, None);
                } else {
                    self.add_token(TokenType::MINUS, None);
                }
            }
            '/' => {
                if self.match_char('>') {
                    self.add_token(TokenType::SLASH_GREATER, None);
                } else if self.match_char('=') {
                    self.add_token(TokenType::SLASH_EQUAL, None);
                } else if self.match_char('<') {
                    self.add_token(TokenType::SLASH_LESS, None);
                } else if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() { self.advance(); }
                } else if self.match_char('*') {
                    self.block_comment();
                } else {
                    self.add_token(TokenType::SLASH, None);
                }
            }
            // Ignore whitespaces.
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.add_token(TokenType::NEWLINE, None);
                self.line += 1;
            }
            '"' => { self.string(); }

            _ => {
                if c.is_digit(10) {
                    self.number();
                } else if c.is_alphabetic() {
                    self.identifier();
                } else {
                    self.error_handler.line_error(self.line, format!("Unexpected character: '{}'.", c));
                }
            }
        }
    }

    //--- Helpers start.

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' { self.advance(); }

        // Check if identifier is reserved.
        let lexeme_slice: String = self.source[self.start..self.current].iter().collect();
        let token_type: TokenType = match KEYWORDS.get(&lexeme_slice) {
            Some(v) => *v,
            None => TokenType::IDENTIFIER,
        };
        self.add_token(token_type, None);
    }

    fn number(&mut self) {
        while self.peek().is_digit(10) { self.advance(); }

        // Check if has fractional part.
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            // Consume the dot.
            self.advance();
            while self.peek().is_digit(10) { self.advance(); }
        }

        let lexeme_slice: String = self.source[self.start..self.current].iter().collect();
        let literal_val: f64 = lexeme_slice.parse().unwrap();
        self.add_token(TokenType::NUMBER, Some(Literals::Number(literal_val)));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' { self.line += 1; }
            self.advance();
        }

        // Unterminated string found.
        if self.is_at_end() {
            self.error_handler.line_error(self.line, "Unterminated string.".to_string());
            return;
        }

        // Consume closing '"'.
        self.advance();

        let literal_val: String = self.source[(self.start + 1)..(self.current - 1)].iter().collect();
        self.add_token(TokenType::STRING, Some(Literals::String(literal_val)));
    }

    fn block_comment(&mut self) {
        while !(self.peek() == '*' && self.peek_next() == '/') && !self.is_at_end() {
            if self.peek() == '\n' { self.line += 1; }
            self.advance();
        }

        // Unterminated block comment found.
        if self.is_at_end() {
            self.error_handler.line_error(self.line, "Unterminated block comment.".to_string());
            return;
        }

        // Consume closing '*/'
        self.current += 2;
    }

    //--- Helpers end.

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literals>) {
        let lexeme_slice: String = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::new(
            token_type,
            lexeme_slice,
            literal,
            self.line
        ))
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() { return false; }
        if self.source[self.current] != expected { return false; }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() { return '\0'; }
        self.source[self.current]
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() { return '\0'; }
        self.source[self.current + 1]
    }
}

lazy_static! {
    static ref KEYWORDS: HashMap<String, TokenType> = {
        let mut m = HashMap::new();
        m.insert("and".to_string(), TokenType::AND);
        m.insert("array".to_string(), TokenType::ARRAY);
        m.insert("break".to_string(), TokenType::BREAK);
        m.insert("class".to_string(), TokenType::CLASS);
        m.insert("continue".to_string(), TokenType::CONTINUE);
        m.insert("dict".to_string(), TokenType::DICT);
        m.insert("else".to_string(), TokenType::ELSE);
        m.insert("false".to_string(), TokenType::FALSE);
        m.insert("fun".to_string(), TokenType::FUN);
        m.insert("for".to_string(), TokenType::FOR);
        m.insert("from".to_string(), TokenType::FROM);
        m.insert("in".to_string(), TokenType::IN);
        m.insert("if".to_string(), TokenType::IF);
        m.insert("lambda".to_string(), TokenType::LAMBDA);
        m.insert("let".to_string(), TokenType::LET);
        m.insert("nil".to_string(), TokenType::NIL);
        m.insert("not".to_string(), TokenType::NOT);
        m.insert("or".to_string(), TokenType::OR);
        m.insert("print".to_string(), TokenType::PRINT);
        m.insert("return".to_string(), TokenType::RETURN);
        m.insert("super".to_string(), TokenType::SUPER);
        m.insert("self".to_string(), TokenType::SELF);
        m.insert("true".to_string(), TokenType::TRUE);
        m.insert("tuple".to_string(), TokenType::TUPLE);
        m.insert("while".to_string(), TokenType::WHILE);
        m
    };
}
