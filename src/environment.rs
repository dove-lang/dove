use std::collections::HashMap;

use crate::token::Literals;
use crate::token::Token;

#[derive(Clone)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    pub values: HashMap<String, Literals>,
    loop_status: LoopStatus,
}

#[derive(Clone)]
enum LoopStatus {
    NotLooping,
    Looping,
    Breaking,
    Continuing,
}

impl Environment {
    pub fn new(enclosing: Option<Box<Environment>>) -> Environment {
        match enclosing {
            Some(enclosing) => Environment{
                enclosing: Some(enclosing),
                values: HashMap::new(),
                loop_status: LoopStatus::NotLooping,
            },
            None => Environment{
                enclosing: None,
                values: HashMap::new(),
                loop_status: LoopStatus::NotLooping
            },
        }
    }

    pub fn get(&self, name: &Token) -> &Literals {
        println!("{:?}", self.values);
        match self.values.get(&name.lexeme) {
            Some(v) => { v }
            None => {
                match &self.enclosing {
                    Some(e) => e.get(name),
                    None => panic!("{} not found in this environment.", name.lexeme),
                }
            }
        }
    }

    pub fn assign(&mut self, name: Token, value: Literals) {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme, value);
        } else {
            match &mut self.enclosing {
                Some(e) => e.assign(name, value),
                None => panic!("{} not found in this environment.", name.lexeme),
            }
        }
    }

    pub fn define(&mut self, name: Token, value: Literals) {
        self.values.insert(name.lexeme, value);
        println!("{:?}", self.values);
    }
}
