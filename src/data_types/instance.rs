use std::rc::Rc;
use std::cell::RefCell;

use crate::data_types::*;
use crate::dove_class::DoveInstance;

impl DoveObject for Rc<RefCell<DoveInstance>> {
    fn get_property(&mut self, name: &str) -> Result<Literals> {
        match DoveInstance::get(Rc::clone(self), name) {
            Some(property) => Ok(property),
            None => Err(Error::CannotGetProperty),
        }
    }

    fn set_property(&mut self, name: &str, value: Literals) -> Result<()> {
        self.borrow_mut().set(name.to_string(), value);
        Ok(())
    }
}
