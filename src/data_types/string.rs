use std::rc::Rc;
use std::cell::RefCell;

use crate::data_types::*;
use crate::dove_callable::{DoveCallable, BuiltinFunction};
use crate::token::Literals;

impl DoveObject for String {
    fn get_property(&mut self, name: &str) -> Result<Literals> {
        match name {
            "len" => Ok(Literals::Function(Rc::new(string_len(self)))),
            "chars" => Ok(Literals::Function(Rc::new(string_chars(self)))),
            _ => Err(Error::CannotGetProperty),
        }
    }
}

fn string_len(string: &str) -> impl DoveCallable {
    let string = string.to_string();

    BuiltinFunction::new(0, move |_| {
        Ok(Literals::Number(string.len() as f64))
    })
}

fn string_chars(string: &str) -> impl DoveCallable {
    let string = string.to_string();

    BuiltinFunction::new(0, move |_| {
        let char_literals = string.chars()
            .map(|c| c.to_string())
            .map(Literals::String)
            .collect();

        Ok(Literals::Array(Rc::new(RefCell::new(char_literals))))
    })
}
