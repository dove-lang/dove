use std::rc::Rc;
use std::cell::RefCell;

use crate::data_types::*;
use crate::error_handler::{RuntimeError, ErrorLocation};
use crate::dove_callable::{DoveCallable, BuiltinFunction};
use crate::token::Literals;

impl DoveObject for Rc<RefCell<Vec<Literals>>> {
    fn get_property(&mut self, name: &str) -> Result<Literals> {
        match name {
            "len" => Ok(Literals::Function(Rc::new(array_len(self)))),
            "is_empty" => Ok(Literals::Function(Rc::new(array_is_empty(self)))),
            "push" => Ok(Literals::Function(Rc::new(array_append(self)))),
            "pop" => Ok(Literals::Function(Rc::new(array_pop(self)))),
            "remove" => Ok(Literals::Function(Rc::new(array_remove(self)))),
            _ => Err(Error::CannotGetProperty),
        }
    }
}

fn array_len(array: &Rc<RefCell<Vec<Literals>>>) -> impl DoveCallable {
    let array = Rc::clone(array);

    BuiltinFunction::new(0, move |_| {
        Ok(Literals::Number(array.borrow().len() as f64))
    })
}

fn array_is_empty(array: &Rc<RefCell<Vec<Literals>>>) -> impl DoveCallable {
    let array = Rc::clone(array);

    BuiltinFunction::new(0, move |_| {
        Ok(Literals::Boolean(array.borrow().len() == 0))
    })
}

fn array_append(array: &Rc<RefCell<Vec<Literals>>>) -> impl DoveCallable {
    let array = Rc::clone(array);

    BuiltinFunction::new(1, move |args| {
        array.borrow_mut().push(args[0].clone());
        Ok(Literals::Nil)
    })
}

fn array_pop(array: &Rc<RefCell<Vec<Literals>>>) -> impl DoveCallable {
    let array = Rc::clone(array);

    BuiltinFunction::new(0, move |_| {
        match array.borrow_mut().pop() {
            Some(v) => Ok(v),
            None => Ok(Literals::Nil),
        }
    })
}

fn array_remove(array: &Rc<RefCell<Vec<Literals>>>) -> impl DoveCallable {
    let array = Rc::clone(array);

    BuiltinFunction::new(1, move |args| {
        let index = match args[0].clone().unwrap_usize() {
            Ok(i) => i,
            _ => return Err(RuntimeError::new(
                ErrorLocation::Unspecified,
                "Index must be an integer.".to_string(),
            )),
        };

        if index >= array.borrow().len() {
            return Err(RuntimeError::new(
                ErrorLocation::Unspecified,
                "Index out of range.".to_string(),
            ));
        }

        Ok(array.borrow_mut().remove(index))
    })
}
