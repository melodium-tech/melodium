
use crate::core::prelude::*;

pub mod bool;
pub mod byte;

pub fn register(mut c: &mut CollectionPool) {

    bool::register(&mut c);
    byte::register(&mut c);
}
