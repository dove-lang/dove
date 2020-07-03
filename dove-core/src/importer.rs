use std::rc::Rc;

use crate::token::{Token, TokenType};
use crate::error_handler::CompiletimeErrorHandler;
use crate::dove::DoveOutput;

pub struct Importer {
    tokens: Vec<Token>,
    error_handler: CompiletimeErrorHandler,

    /// When `expecting_file_name` is false, look for the `import` keyword;
    /// when it is true, look for Token with TokenType::String.
    expecting_file_name: bool,

    current: usize,
}

impl Importer {
    pub fn new(tokens: Vec<Token>, output: Rc<dyn DoveOutput>) -> Importer {
        Importer {
            tokens,
            error_handler: CompiletimeErrorHandler {
                had_error: false,
                output,
            },
            expecting_file_name: false,
            current: 0,
        }
    }

    pub fn analyze(&mut self) -> (Vec<Token>, Vec<String>) {
        let mut imports: Vec<String> = Vec::new();

        // Scan for import strings.
        while !self.is_at_end() {
            if self.check(vec![TokenType::IMPORT, TokenType::STRING, TokenType::NEWLINE]) {

                // Looking for an `Import` Token.
                if !self.expecting_file_name {
                    let token = self.advance();
                    match &token.token_type {
                        TokenType::IMPORT => {
                            self.expecting_file_name = true;
                        },
                        TokenType::STRING => { break; }
                        // NEWLINE's, ignore.
                        _ => {}
                    }
                }
                // Looking for a file dir string.
                else {
                    let token = self.advance();
                    match &token.token_type {
                        TokenType::STRING => {
                            // Remove leading and trailing '"'.
                            let mut path = token.lexeme;
                            path.truncate(path.len() - 1);
                            path.drain(..1);

                            imports.push(path);
                            self.expecting_file_name = false;
                        },
                        TokenType::IMPORT => {
                            self.handle_error(token, "Expecting a file name after 'import' keyword.".to_string());
                            break;
                        },
                        // NEWLINE's, ignore.
                        _ => {}
                    }
                }

            } else {
                // If expecting a file name string, report error.
                if self.expecting_file_name {
                    let token = self.advance();
                    self.handle_error(token, "Expecting a file name after 'import' keyword.".to_string());
                }

                break
            }
        }

        // Remove any consumed tokens.
        self.tokens.drain(..self.current);

        (self.tokens.clone(), imports)
    }

    fn handle_error(&mut self, token: Token, message: String) {
        self.error_handler.token_error(token, message);
    }
}

// Helpers.
impl Importer {
    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> Token {
        let token = self.peek().clone();

        if !self.is_at_end() {
            self.current += 1;
        }

        token
    }

    /// Returns true if current token is one of the given TokenType's.
    fn check(&self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types {
            if !self.is_at_end() && self.peek().token_type == token_type {
                return true
            }
        }

        false
    }
}
