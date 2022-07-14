
pub mod read;
pub mod write;

use crate::core::prelude::*;

pub fn register(mut c: &mut CollectionPool) {

    read::register(&mut c);
    write::register(&mut c);
}
