
use crate::core::prelude::*;

pub mod static_filling;

pub fn register(mut c: &mut CollectionPool) {
    static_filling::register(&mut c);
}
