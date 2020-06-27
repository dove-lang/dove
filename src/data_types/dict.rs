use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::data_types::*;
use crate::dove_callable::{DoveCallable, BuiltinFunction};
use crate::token::{Literals, DictKey};

impl DoveObject for Rc<RefCell<HashMap<DictKey, Literals>>> {
    fn get_property(&mut self, name: &str) -> Result<Literals> {
        match name {
            "length" => Ok(Literals::Number(self.borrow().len() as f64)),
            "keys" => Ok(Literals::Function(Rc::new(dict_keys(self)))),
            "values" => Ok(Literals::Function(Rc::new(dict_values(self)))),
            "remove" => Ok(Literals::Function(Rc::new(dict_remove(self)))),
            _ => Err(Error::CannotGetProperty),
        }
    }
}

fn dict_keys(dict: &Rc<RefCell<HashMap<DictKey, Literals>>>) -> impl DoveCallable {
    let dict = Rc::clone(dict);

    BuiltinFunction::new(0, move |_| {
        let mut res_raw = Vec::new();

        for key in dict.borrow().keys() {
            match key.clone() {
                DictKey::StringKey(s) => res_raw.push(Literals::String(s)),
                DictKey::NumberKey(n) => res_raw.push(Literals::Number(n as f64)),
            }
        }

        Literals::Array(Rc::new(RefCell::new(res_raw)))
    })
}

fn dict_values(dict: &Rc<RefCell<HashMap<DictKey, Literals>>>) -> impl DoveCallable {
    let dict = Rc::clone(dict);

    BuiltinFunction::new(0, move |_| {
        let mut res_raw = Vec::new();

        for val in dict.borrow().values() {
            res_raw.push(val.clone());
        }

        Literals::Array(Rc::new(RefCell::new(res_raw)))
    })
}

fn dict_remove(dict: &Rc<RefCell<HashMap<DictKey, Literals>>>) -> impl DoveCallable {
    let dict = Rc::clone(dict);

    BuiltinFunction::new(1, move |args| {
        let key = args[0].clone();

        // Convert key to DictKey type.
        let dict_key = match key {
            Literals::String(s) => DictKey::StringKey(s),
            Literals::Number(n) => {
                if n.fract() != 0.0 { panic!("Expected an integer key. "); }
                DictKey::NumberKey(n as usize)
            },
            _ => {
                panic!("Expected a string or an integer key.")
            }
        };

        match dict.borrow_mut().remove(&dict_key) {
            Some(v) => v,
            None => Literals::Nil,
        }
    })
}
