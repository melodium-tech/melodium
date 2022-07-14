

pub mod direct;

pub mod files;
pub mod read_files;
pub mod write_files;

pub mod read;
pub mod write;

use crate::core::prelude::*;

pub fn register(mut c: &mut CollectionPool) {

    direct::register(&mut c);

    read::register(&mut c);
    write::register(&mut c);
}
