use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::dove_callable::DoveFunction;
use crate::token::Literals;

#[derive(Debug)]
pub struct DoveClass {
    name: String,
    methods: HashMap<String, Rc<DoveFunction>>,
}

impl DoveClass {
    pub fn new(name: String, methods: HashMap<String, Rc<DoveFunction>>) -> DoveClass {
        DoveClass {
            name,
            methods,
        }
    }

    fn find_method(&self, name: &String) -> Option<Rc<DoveFunction>> {
        self.methods.get(name).map(Rc::clone)
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

    pub fn get(instance: Rc<RefCell<DoveInstance>>, field: &String) -> Option<Literals> {
        let instance_ref = instance.borrow();
        instance_ref.fields.get(field).map(Literals::clone)
            .or_else(|| {
                // TODO: cache binded methods?
                instance_ref.class.find_method(field).map(|method| {
                    let method = Rc::new(method.bind(Rc::clone(&instance)));
                    Literals::Function(method)
                })
            })
    }

    // pub fn get(&self, field: &String) -> Option<Literals> {
    //     self.fields.get(field).map(Clone::clone)
    //         .or_else(|| self.class.find_method(field).map(|method| Literals::Function(method)))
    // }

    pub fn set(&mut self, field: String, value: Literals) {
        self.fields.insert(field, value);
    }
}
