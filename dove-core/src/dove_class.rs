use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::dove_callable::DoveFunction;
use crate::token::Literals;

#[derive(Debug)]
pub struct DoveClass {
    pub name: String,
    superclass: Option<Rc<DoveClass>>,
    methods: HashMap<String, Rc<DoveFunction>>,
}

impl DoveClass {
    pub fn new(name: String, superclass: Option<Rc<DoveClass>>, methods: HashMap<String, Rc<DoveFunction>>) -> DoveClass {
        DoveClass {
            name,
            superclass,
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<DoveFunction>> {
        if let Some(method) = self.methods.get(name) {
            Some(Rc::clone(&method))
        } else if let Some(superclass) = &self.superclass {
            superclass.find_method(name)
        } else {
            None
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
        DoveInstance {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(instance: Rc<RefCell<DoveInstance>>, field: &str) -> Option<Literals> {
        let mut instance_ref = instance.borrow_mut();

        match instance_ref.fields.get(field) {
            Some(value) => Some(value.clone()),
            None => {
                instance_ref.class.find_method(field).map(|method| {
                    let bound_method = method.bind(Rc::clone(&instance));
                    let literal = Literals::Function(Rc::new(bound_method));

                    // Lazily bind method and save to fields
                    instance_ref.set(field.to_string(), literal.clone());

                    literal
                })
            }
        }
    }

    pub fn set(&mut self, field: String, value: Literals) {
        self.fields.insert(field, value);
    }
}
