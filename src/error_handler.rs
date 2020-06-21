use crate::token::*;

/// All ErrorHandlers should implement this trait
/// and use its `report` method to display error messages.
pub trait ErrorHandler {
    fn report(&mut self, line: usize, where_: String, message: String) {
        e_red_ln!("[line {}] Error{}: {}", line, where_, message);
    }
}

pub struct RuntimeErrorHandler {
    pub had_runtime_error: bool,
}

impl RuntimeErrorHandler {
    pub fn new() -> RuntimeErrorHandler {
        RuntimeErrorHandler {
            had_runtime_error: false,
        }
    }

    pub fn runtime_error(&mut self, error: RuntimeError) {
        self.had_runtime_error = true;
        self.report(error.token.line, "".to_string(), error.message);
    }
}

impl ErrorHandler for RuntimeErrorHandler {}

pub struct CompiletimeErrorHandler {
    pub had_error: bool,
}

impl CompiletimeErrorHandler {
    pub fn new() -> CompiletimeErrorHandler {
        CompiletimeErrorHandler {
            had_error: false,
        }
    }

    pub fn line_error(&mut self, line: usize, message: String) {
        self.had_error = true;
        self.report(line, "".to_string(), message);
    }

    pub fn token_error(&mut self, token: Token, message: String) {
        self.had_error = true;
        match token.token_type {
            TokenType::EOF => self.report(token.line, " at end".to_string(), message),
            _ => self.report(token.line, format!(" at '{}'", token.lexeme), message),
        }
    }
}

impl ErrorHandler for CompiletimeErrorHandler {}


/// RuntimeError struct used to structure information of
/// a runtime error.
pub struct RuntimeError {
    token: Token,
    message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: String) -> Self {
        RuntimeError {
            token,
            message,
        }
    }
}
