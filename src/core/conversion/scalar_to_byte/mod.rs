
use crate::core::prelude::*;

mod number_to_byte;
mod bool_to_byte;
mod char_to_byte;
mod new_char_to_byte;
mod string_to_byte;

pub fn register(c: &mut CollectionPool) {

    bool_to_byte::bool_to_byte::register(&mut c);
    char_to_byte::char_to_byte::register(&mut c);

    number_to_byte::register(c);

    string_to_byte::string_to_byte::register(&mut c);

    //c.treatments.insert(&(bool_to_byte::BoolToByte::descriptor() as Arc<dyn TreatmentDescriptor>));
    //c.treatments.insert(&(char_to_byte::CharToByte::descriptor() as Arc<dyn TreatmentDescriptor>));
    //c.treatments.insert(&(string_to_byte::StringToByte::descriptor() as Arc<dyn TreatmentDescriptor>));
}

