
use crate::core::prelude::*;

pub mod block_static_filling;
pub mod filling;
pub mod static_filling;

pub fn register(mut c: &mut CollectionPool) {
    block_static_filling::register(&mut c);
    filling::register(&mut c);
    static_filling::register(&mut c);
}
