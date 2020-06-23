use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

use crate::token::Literals;
use crate::token::Token;

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

    pub fn get(&self, name: &Token) -> Result<Literals, ()> {
        match self.values.get(&name.lexeme) {
            Some(v) => Ok(v.clone()),
            None => {
                match &self.enclosing {
                    Some(e) => e.borrow().get(name),
                    None => Err(()),
                }
            }
        }
    }

    pub fn assign(&mut self, name: Token, value: Literals) -> Result<(), ()> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme, value);
            Ok(())
        } else {
            match &mut self.enclosing {
                Some(e) => e.borrow_mut().assign(name, value),
                None => Err(()),
            }
        }
    }

    pub fn define(&mut self, name: Token, value: Literals) {
        self.values.insert(name.lexeme, value);
    }
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Environment")
            .finish()
    }
}
