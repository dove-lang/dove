use std::fs::File;
use std::{ io, process };
use std::io::{ErrorKind, Read, Write};

use chrono::prelude::*;

use crate::scanner::Scanner;
use crate::importer::Importer;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::resolver::Resolver;

pub struct Dove {
    interpreter: Interpreter,
    is_repl_unfinished: bool,

    /// Keep track of what files this Dove has visited.
    visited_imports: Vec<String>,
}

impl Dove {
    pub fn new() -> Self {
        Dove {
            interpreter: Interpreter::new(),
            is_repl_unfinished: false,
            visited_imports: Vec::new(),
        }
    }

    pub fn run_file(mut self, path: &String) -> Self {
        let mut f = match File::open(path) {
            Ok(file) => file,
            Err(error) => match error.kind() {
                ErrorKind::NotFound => {
                    e_red_ln!("File: '{}' not found.", path);
                    process::exit(53);
                },
                _ => {
                    e_red_ln!("Error while reading file: {} {:?}", path, error);
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

        self = self.run(content.chars().collect(), false);

        self
    }

    pub fn run_prompt(mut self) {
        // Print version & time information.
        let date = Local::now();
        cyan_ln!("Dove 0.1.1 (default, {})", date.format("%b %e %Y, %H:%M:%S"));
        cyan_ln!("Visit https://github.com/dove-lang for more information.");

        // Used to store previous lines of code, if encounters unfinished blocks.
        let mut code_buffer = String::new();

        loop {
            let indicator = format!("{} ", if self.is_repl_unfinished {"..."} else {">>>"});
            print!("{}", indicator);

            let mut input = String::new();
            // `stdout` gets flushed on new lines, manually flush it.
            let _ = io::stdout().flush();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {},
                Err(_) => {
                    e_red_ln!("Error while reading input to string.");
                    process::exit(92);
                }
            }

            let input = format!("{}{}", code_buffer, input);

            self = self.run(input.chars().collect(), true);

            // If Dove is in an unfinished block, store `input` back in `code_buffer`,
            // otherwise clear `code_buffer`.
            if self.is_repl_unfinished {
                code_buffer = input;
            } else {
                code_buffer = String::new();
            }

            // Reset the flag; one mistake from the user shouldn't kill the entire session.
            // self.had_error = false;
        }
    }

    fn run(mut self, source: Vec<char>, is_in_repl: bool) -> Self {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        let mut importer = Importer::new(tokens);
        let (tokens, imports) = importer.analyze();

        // Run the import files.
        for import in imports {
            if self.visited_imports.contains(&import) {
                e_red_ln!("Import Error: Cannot import file '{}'.", import);
                process::exit(92);
            }

            self.visited_imports.push(import.clone());
            self = self.run_file(&import);
        }

        let mut parser = Parser::new(tokens, is_in_repl);
        let statements = parser.program();

        // Check if unfinished status change.
        if parser.is_in_unfinished_blk != self.is_repl_unfinished {
            self.is_repl_unfinished = !self.is_repl_unfinished;
        }

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
