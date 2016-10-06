#![allow(unused_variables)]

use parser;
pub struct Null;

impl Null {
    pub fn new() -> Null {
        Null
    }
}

// Inherits the defaults
impl parser::HasHandlers for Null {}
