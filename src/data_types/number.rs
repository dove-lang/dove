use std::rc::Rc;

use crate::data_types::*;
use crate::dove_callable::{DoveCallable, BuiltinFunction};
use crate::token::Literals;

impl DoveObject for f64 {
    fn get_property(&mut self, name: &str) -> Result<Literals> {
        match name {
            "fract" => Ok(Literals::Function(Rc::new(number_fract(*self)))),
            "abs" => Ok(Literals::Function(Rc::new(number_abs(*self)))),
            "floor" => Ok(Literals::Function(Rc::new(number_floor(*self)))),
            "ceil" => Ok(Literals::Function(Rc::new(number_ceil(*self)))),
            _ => Err(Error::CannotGetProperty),
        }
    }
}

fn number_fract(number: f64) -> impl DoveCallable {
    BuiltinFunction::new(0, move |_| {
        Ok(Literals::Number(number.fract()))
    })
}

fn number_abs(number: f64) -> impl DoveCallable {
    BuiltinFunction::new(0, move |_| {
        Ok(Literals::Number(number.abs()))
    })
}

fn number_floor(number: f64) -> impl DoveCallable {
    BuiltinFunction::new(0, move |_| {
        Ok(Literals::Number(number.floor()))
    })
}

fn number_ceil(number: f64) -> impl DoveCallable {
    BuiltinFunction::new(0, move |_| {
        Ok(Literals::Number(number.ceil()))
    })
}

