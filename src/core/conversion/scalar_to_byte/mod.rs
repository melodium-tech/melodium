
use crate::core::prelude::*;

mod number_to_byte;
mod bool_to_byte;
mod char_to_byte;
mod string_to_byte;

pub fn register(c: &mut CollectionPool) {

    bool_to_byte::bool_to_byte::register(&mut c);
    char_to_byte::char_to_byte::register(&mut c);

    number_to_byte::register(c);

    string_to_byte::string_to_byte::register(&mut c);
}

