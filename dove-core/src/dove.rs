use std::rc::Rc;

use crate::scanner::Scanner;
use crate::importer::Importer;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::resolver::Resolver;

pub trait DoveOutput {
    fn print(&self, message: String);
    fn warning(&self, message: String);
    fn error(&self, message: String);
}

pub struct Dove {
    interpreter: Interpreter,
    pub is_repl_unfinished: bool,

    /// Keep track of what files this Dove has visited.
    visited_imports: Vec<String>,

    output: Rc<dyn DoveOutput>,
}

impl Dove {
    pub fn new(output: Rc<dyn DoveOutput>) -> Self {
        Dove {
            interpreter: Interpreter::new(Rc::clone(&output)),
            is_repl_unfinished: false,
            visited_imports: Vec::new(),
            output,
        }
    }

    pub fn run(&mut self, source: Vec<char>, is_in_repl: bool) {
        let scanner = Scanner::new(source, Rc::clone(&self.output));
        let tokens = scanner.scan_tokens();

        let mut importer = Importer::new(tokens, Rc::clone(&self.output));
        let (tokens, imports) = importer.analyze();

        // Run the import files.
        // TODO: how to handle import in wasm?
        // for import in imports {
        //     if self.visited_imports.contains(&import) {
        //         e_red_ln!("Import Error: Cannot import file '{}'.", import);
        //         process::exit(92);
        //     }

        //     self.visited_imports.push(import.clone());
        //     self = self.run_file(&import);
        // }

        let mut parser = Parser::new(tokens, is_in_repl, Rc::clone(&self.output));
        let statements = parser.program();

        // Check if unfinished status change.
        if parser.is_in_unfinished_blk != self.is_repl_unfinished {
            self.is_repl_unfinished = !self.is_repl_unfinished;
        }

        // Stops if there is a syntax error.
        // if self.had_error {
        //     return self;
        // }

        let mut resolver = Resolver::new(&mut self.interpreter, Rc::clone(&self.output));
        resolver.resolve(&statements);

        self.interpreter.interpret(statements);
    }
}
