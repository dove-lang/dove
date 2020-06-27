use std::rc::Rc;
use std::cell::RefCell;

use crate::data_types::*;
use crate::dove_callable::{DoveCallable, BuiltinFunction};
use crate::token::Literals;

impl DoveObject for Rc<RefCell<Vec<Literals>>> {
    fn get_property(&mut self, name: &str) -> Result<Literals> {
        match name {
            "length" => Ok(Literals::Number(self.borrow().len() as f64)),
            "empty" => Ok(Literals::Boolean(self.borrow().len() == 0)),
            "push" => Ok(Literals::Function(Rc::new(array_append(self)))),
            "pop" => Ok(Literals::Function(Rc::new(array_pop(self)))),
            "remove" => Ok(Literals::Function(Rc::new(array_remove(self)))),
            _ => Err(Error::CannotGetProperty),
        }
    }
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
        // TODO: Add better error handling.
        let index = args[0].clone().unwrap_number().unwrap_or_else(|_| {
            panic!("Invalid index.");
        });

        if index.fract() != 0.0 || index < 0.0 {
            panic!("Invalid index.");
        }

        let index = index as usize;
        if index >= array.borrow().len() {
            panic!("Invalid index.");
        }

        Ok(array.borrow_mut().remove(index))
    })
}
