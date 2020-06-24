use std::rc::Rc;
use std::collections::HashMap;

use crate::token::Literals;

#[derive(Debug)]
pub struct DoveClass {
    name: String,
}

impl DoveClass {
    pub fn new(name: String) -> DoveClass {
        DoveClass {
            name,
        }
    }
}

#[derive(Debug)]
pub struct DoveInstance {
    class: Rc<DoveClass>,
    fields: HashMap<String, Literals>,
}

impl DoveInstance {
    pub fn new(class: Rc<DoveClass>) -> DoveInstance {
        // TODO: run initializer?
        DoveInstance {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, field: &String) -> Option<&Literals> {
        self.fields.get(field)
    }

    pub fn set(&mut self, field: String, value: Literals) {
        self.fields.insert(field, value);
    }
}
