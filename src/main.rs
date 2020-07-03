#[macro_use(e_red_ln, e_yellow_ln, cyan_ln)]
extern crate colour;

use std::env;

use std::fs::File;
use std::{ io, process };
use std::io::{ErrorKind, Read, Write};
use std::rc::Rc;

use chrono::prelude::*;

use dove_core::dove::{Dove, DoveOutput};

struct Output;
impl DoveOutput for Output {
    fn print(&self, message: String) {
        println!("{}", message);
    }

    fn warning(&self, message: String) {
        e_yellow_ln!("{}", message);
    }

    fn error(&self, message: String) {
        e_red_ln!("{}", message);
    }
}

fn main() {
    // Collect command line arguments.
    // Note: The first value is always the name of the binary.
    let args: Vec<String> = env::args().collect();
    let mut dove = Dove::new(Rc::new(Output {}));

    if args.len() > 2 {
        println!("Usage: dove [script]");
    } else if args.len() == 2 {
        let path = &args[1];
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

        dove.run(content.chars().collect(), false);

    } else {
        // Print version & time information.
        let date = Local::now();
        cyan_ln!("Dove 0.1.1 (default, {})", date.format("%b %e %Y, %H:%M:%S"));
        cyan_ln!("Visit https://github.com/dove-lang for more information.");

        // Used to store previous lines of code, if encounters unfinished blocks.
        let mut code_buffer = String::new();

        loop {
            let indicator = format!("{} ", if dove.is_repl_unfinished {"..."} else {">>>"});
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

            dove.run(input.chars().collect(), true);

            // If Dove is in an unfinished block, store `input` back in `code_buffer`,
            // otherwise clear `code_buffer`.
            if dove.is_repl_unfinished {
                code_buffer = input;
            } else {
                code_buffer = String::new();
            }

            // Reset the flag; one mistake from the user shouldn't kill the entire session.
            // self.had_error = false;
        }
    }
}
