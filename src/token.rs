#[derive(Clone)]
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
    COMMA, DOT, MINUS, NEWLINE, PLUS, SLASH, STAR,

    // One or two character tokens.
    BACKSLASH,
    BANG, BANG_EQUAL,
    EQUAL, EQUAL_EQUAL,
    GREATER, GREATER_EQUAL,
    LESS, LESS_EQUAL,

    // Literals.
    IDENTIFIER, STRING, NUMBER,

    // Keywords.
    AND, ARRAY, BREAK, CLASS, CONTINUE, DICT, ELSE, FALSE, FUN, FOR, FROM, IN, IF, LET, NIL, NOT, OR,
    PRINT, RETURN, SUPER, SELF, TRUE, TUPLE, WHILE,

    // End of file.
    EOF
}

#[derive(Debug, Clone)]
pub enum Literals {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}
