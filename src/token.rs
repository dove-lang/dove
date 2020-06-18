pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literals,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Literals, line: usize) -> Token {
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

#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN, RIGHT_PAREN, LEFT_BRACE, RIGHT_BRACE, LEFT_BRACKET, RIGHT_BRACKET,
    COMMA, DOT, MINUS, NEWLINE, PLUS, SLASH, STAR,

    // One or two character tokens.
    BANG, BANG_EQUAL,
    EQUAL, EQUAL_EQUAL,
    GREATER, GREATER_EQUAL,
    LESS, LESS_EQUAL,

    // Literals.
    IDENTIFIER, STRING, NUMBER,

    // Keywords.
    AND, BREAK, CLASS, ELSE, FALSE, FUN, FOR, IN, IF, LET, NIL, NOT, OR,
    PRINT, RETURN, SUPER, SELF, TRUE, WHILE,

    // End of file.
    EOF
}

#[derive(Debug)]
pub enum Literals {
    STRING(String),
    NUMBER(f64),
    BOOLEAN(bool),
    NULL,
}
