use std::rc::Rc;
use std::cell::RefCell;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Array;

use dove_core::{Scanner, Interpreter, Parser, Resolver, DoveOutput};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(typescript_type = "Array<string>")]
    pub type StringArray;
}

struct Output {
    lines: RefCell<Vec<String>>,
}

impl Output {
    fn new() -> Output {
        Output {
            lines: RefCell::new(vec![]),
        }
    }
}

impl DoveOutput for Output {
    fn print(&self, message: String) {
        self.lines.borrow_mut().push(message);
    }

    fn warning(&self, message: String) {
        self.lines.borrow_mut().push(message);
    }

    fn error(&self, message: String) {
        self.lines.borrow_mut().push(message);
    }
}

/// Run the source and return the output as an array of strings.
#[wasm_bindgen]
pub fn run(source: String) -> StringArray {
    let output_raw = Rc::new(Output::new());
    let output = Rc::clone(&output_raw) as Rc<dyn DoveOutput>;

    let chars = source.chars().collect();
    let scanner = Scanner::new(chars, Rc::clone(&output));
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens, false, Rc::clone(&output));
    let statements = parser.program();

    // Stops if there is a syntax error.
    // if self.had_error {
    //     return self;
    // }
    let mut interpreter = Interpreter::new(Rc::clone(&output));

    let mut resolver = Resolver::new(&mut interpreter, Rc::clone(&output));
    resolver.resolve(&statements);

    interpreter.interpret(statements);

    let str_arr = output_raw.lines.borrow().iter()
        .map(JsValue::from)
        .collect::<Array>()
        .unchecked_into::<StringArray>();

    str_arr
}
