use std::env;
use std::fs::File;
use std::io;
use std::io::{ErrorKind, Read, Write};

use dove::scanner::*;
use dove::token::*;

fn main() {
    // Collect command line arguments.
    // Note: The first value is always the name of the binary.
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: dove [script]");
    } else if args.len() == 2 {
        run_file(&args[1]).expect(format!("Failed to open file: {}", &args[1]).as_ref());
    } else {
        run_prompt();
    }
}

fn run_file(path: &String) -> io::Result<()> {
    let mut f = match File::open(&path) {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => { panic!("File not found: {:?}", error) },
            other_error => { panic!("Problem opening the file: {:?}", other_error) }
        },
    };

    let mut content = String::new();
    f.read_to_string(&mut content).expect("Error when reading file.");

    run(content);

    Ok(())
}

fn run_prompt() {
    loop {
        let mut input = String::new();
        print!(">>> ");

        // `stdout` gets flushed on new lines. Manually flush it.
        let _ = io::stdout().flush();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {},
            Err(error) => println!("error: {}", error),
        }

        run(input);
    }
}

fn run(source: String) {
    let scanner = Scanner::new(source);
    let tokens: Vec<Token> = scanner.scan_tokens();

    for token in tokens.iter() {
        println!("{}", token.to_string());
    }
}
