use std::env;

use dove::dove::Dove;

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
