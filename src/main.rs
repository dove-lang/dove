#[macro_use(e_red_ln, e_yellow_ln, cyan_ln)]
extern crate colour;

mod dove;

use std::env;
use std::rc::Rc;

use dove_core::dove_output::DoveOutput;
use dove::Dove;

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
        dove.run_file(&args[1]);
    } else {
        dove.run_prompt();
    }
}
