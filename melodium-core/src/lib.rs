#[macro_use]
extern crate lazy_static;

pub use melodium_common as common;
pub mod descriptor;

#[allow(non_camel_case_types)]
pub type byte = u8;
#[allow(non_camel_case_types)]
pub type string = String;
#[allow(non_camel_case_types)]
pub type void = ();
