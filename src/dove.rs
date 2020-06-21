use std::fs::File;
use std::{ io, process };
use std::io::{ErrorKind, Read, Write};

use crate::scanner::*;
use crate::token::*;
use crate::interpreter::Interpreter;
use crate::ast::Expr;
use crate::parser::Parser;

pub struct Dove {
    interpreter: Interpreter,
    had_error: bool,
    had_runtime_error: bool,
}

impl Dove {
    pub fn new() -> Self {
        Dove {
            interpreter: Interpreter::new(),
            had_error: false,
            had_runtime_error: false,
        }
    }

    pub fn run_file(mut self, path: &String) {
        let mut f = match File::open(path) {
            Ok(file) => file,
            Err(error) => match error.kind() {
                ErrorKind::NotFound => {
                    e_red_ln!("File: '{}' not found.", path);
                    process::exit(53);
                },
                _ => {
                    e_red_ln!("Error while reading file: {}", path);
                    process::exit(75);
                }
            }
        };

        let mut content = String::new();
        match f.read_to_string(&mut content) {
            Ok(_) => {},
            Err(e) => {
                e_red_ln!("Error while reading file '{}' to string.", path);
                process::exit(92);
            }
        }

        self.run(content.chars().collect());
    }

    pub fn run_prompt(mut self) {
        loop {
            let mut input = String::new();
            print!(">>> ");

            // `stdout` gets flushed on new lines, manually flush it.
            let _ = io::stdout().flush();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {},
                Err(error) => {
                    e_red_ln!("Error while reading input to string.");
                    process::exit(92);
                }
            }

            self = self.run(input.chars().collect());

            // Reset the flag; one mistake from the user shouldn't kill the entire session.
            self.had_error = false;
        }
    }

    fn run(mut self, source: Vec<char>) -> Self {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens.to_owned());
        let statements = parser.program().unwrap_or_default();

        // Stops if there is a syntax error.
        if self.had_error {
            return self;
        }

        self.interpreter.interpret(statements);
        self
    }
}
