use crate::token::*;

/// All ErrorHandlers should implement this trait
/// and use its `report` method to display error messages.
pub trait ErrorHandler {
    fn report(&mut self, line: Option<usize>, where_: String, message: String) {
        if let Some(line) = line {
            e_red_ln!("[line {}] Error{}: {}", line, where_, message);
        } else {
            e_red_ln!("Error: {}",message);
        }
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
        self.report(
            error.location.line(),
            match error.location {
                ErrorLocation::Token(token) => format!(" at '{}'", token.lexeme),
                _ => "".to_string(),
            },
            error.message,
        );
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
        self.report(Some(line), "".to_string(), message);
    }

    pub fn token_error(&mut self, token: Token, message: String) {
        self.had_error = true;
        match token.token_type {
            TokenType::EOF => self.report(Some(token.line), " at end".to_string(), message),
            _ => self.report(Some(token.line), format!(" at '{}'", token.lexeme), message),
        }
    }
}

impl ErrorHandler for CompiletimeErrorHandler {}

#[derive(Debug, Clone)]
pub enum ErrorLocation {
    Token(Token),
    Line(usize),
    // TODO: maybe remove this after adding token to all ASTs
    Unspecified,
}

impl ErrorLocation {
    pub fn line(&self) -> Option<usize> {
        match self {
            ErrorLocation::Token(token) => Some(token.line),
            ErrorLocation::Line(line) => Some(*line),
            _ => None,
        }
    }
}

/// RuntimeError struct used to structure information of
/// a runtime error.
#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub location: ErrorLocation,
    pub message: String,
}

impl RuntimeError {
    pub fn new(location: ErrorLocation, message: String) -> Self {
        RuntimeError {
            location,
            message,
        }
    }
}
