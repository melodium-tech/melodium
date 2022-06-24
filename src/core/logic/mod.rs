
use crate::core::prelude::*;

pub mod bool;

pub fn register(mut c: &mut CollectionPool) {

    bool::register(&mut c);
}
