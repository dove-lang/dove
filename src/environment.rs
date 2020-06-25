use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

use crate::token::Literals;

#[derive(Clone)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Literals>,
    pub loop_status: LoopStatus,
}

#[derive(Clone)]
pub enum LoopStatus {
    NotLooping,
    Looping,
    Breaking,
    Continuing,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Environment {
        Environment{
            enclosing: enclosing,
            values: HashMap::new(),
            loop_status: LoopStatus::NotLooping,
        }
    }

    pub fn get(&self, name: &str) -> Option<Literals> {
        self.values.get(name).map(Literals::clone)
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Option<Literals> {
        if distance <= 0 {
            self.get(name)
        } else {
            match &self.enclosing {
                Some(enclosing) => enclosing.borrow().get_at(distance - 1, name),
                None => None,
            }
        }
    }

    pub fn assign(&mut self, name: String, value: Literals) -> bool {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            true
        } else {
            false
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: String, value: Literals) -> bool {
        if distance <= 0 {
            self.assign(name, value)
        } else {
            match &self.enclosing {
                Some(enclosing) => enclosing.borrow_mut().assign_at(distance - 1, name, value),
                None => false,
            }
        }
    }

    pub fn define(&mut self, name: String, value: Literals) {
        self.values.insert(name, value);
    }
}

// Scope debugging functions
impl Environment {
    pub fn hierarchy(&self, count: usize) -> String {
        let string = format!("{}{}", " ".repeat(4 * count), self.vars());
        match &self.enclosing {
            Some(encl) => format!("{}\n{}", string, encl.borrow().hierarchy(count + 1)),
            None => string,
        }
    }

    fn vars(&self) -> String {
        let mut string = "{ ".to_string();
        for (key, _) in self.values.iter() {
            string.push_str(&format!("{}, ", key));
        }
        string.push_str(" }");
        string
    }
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Environment")
            .finish()
    }
}
