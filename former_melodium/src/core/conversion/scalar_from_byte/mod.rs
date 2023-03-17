use crate::core::prelude::*;

mod bool_from_byte;
mod char_from_byte;
mod number_from_byte;
mod string_from_byte;

pub fn register(mut c: &mut CollectionPool) {

    bool_from_byte::bool_from_byte::register(&mut c);
    char_from_byte::char_from_byte::register(&mut c);

    number_from_byte::register(c);

    string_from_byte::string_from_byte::register(&mut c);
}
