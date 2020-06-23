use std::fs::File;
use std::{ io, process };
use std::io::{ErrorKind, Read, Write};

use chrono::prelude::*;

use crate::scanner::*;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::resolver::Resolver;

pub struct Dove {
    interpreter: Interpreter,
}

impl Dove {
    pub fn new() -> Self {
        Dove {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file(self, path: &String) {
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
            Err(_) => {
                e_red_ln!("Error while reading file '{}' to string.", path);
                process::exit(92);
            }
        }

        self.run(content.chars().collect());
    }

    pub fn run_prompt(mut self) {
        // Print version & time information.
        let date = Local::now();
        cyan_ln!("Dove 0.0.1 (default, {})", date.format("%b %e %Y, %H:%M:%S"));
        cyan_ln!("Visit https://github.com/dove-lang for more information.");

        loop {
            let mut input = String::new();
            print!(">>> ");

            // `stdout` gets flushed on new lines, manually flush it.
            let _ = io::stdout().flush();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {},
                Err(_) => {
                    e_red_ln!("Error while reading input to string.");
                    process::exit(92);
                }
            }

            self = self.run(input.chars().collect());

            // Reset the flag; one mistake from the user shouldn't kill the entire session.
            // self.had_error = false;
        }
    }

    fn run(mut self, source: Vec<char>) -> Self {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens.to_owned());
        let statements = parser.program();

        // Stops if there is a syntax error.
        // if self.had_error {
        //     return self;
        // }

        let mut resolver = Resolver::new(&mut self.interpreter);
        resolver.resolve(&statements);

        self.interpreter.interpret(statements);
        self
    }
}
