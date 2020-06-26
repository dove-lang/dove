use crate::token::Literals;

// TODO: add more errors?
pub enum Error {
    PropertyNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait DoveObject {
    fn get_property(&mut self, name: &str) -> Result<Literals>;
    fn set_property(&mut self, name: String, value: Literals) -> Result<()>;
}
