use crate::token::Literals;

pub mod string;
pub mod instance;

// TODO: add more errors?
pub enum Error {
    CannotGetProperty,
    CannotSetProperty,
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait DoveObject {
    fn get_property(&mut self, name: &str) -> Result<Literals> {
        Err(Error::CannotGetProperty)
    }

    fn set_property(&mut self, name: &str, value: Literals) -> Result<()> {
        Err(Error::CannotSetProperty)
    }
}
