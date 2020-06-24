#[macro_use(cyan_ln, e_red_ln)]
extern crate colour;
extern crate chrono;

pub mod dove;

pub mod scanner;
pub mod token;
pub mod ast;
pub mod dove_callable;
pub mod interpreter;
pub mod environment;
pub mod parser;
pub mod error_handler;
pub mod resolver;
pub mod dove_class;
