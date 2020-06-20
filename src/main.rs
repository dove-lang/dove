use std::env;
use std::fs::File;
use std::io;
use std::io::{ErrorKind, Read, Write};

use dove::dove::Dove;

use dove::scanner::*;
use dove::token::*;
use dove::interpreter::Interpreter;
use dove::parser::Parser;


fn main() {
    // Collect command line arguments.
    // Note: The first value is always the name of the binary.
    let args: Vec<String> = env::args().collect();
    let dove = Dove::new();

    if args.len() > 2 {
        println!("Usage: dove [script]");
    } else if args.len() == 2 {
        dove.run_file(&args[1]);
    } else {
        dove.run_prompt();
    }
}
