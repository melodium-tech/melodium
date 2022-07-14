

pub mod direct;

pub mod files;
pub mod read_files;
pub mod write_files;

pub mod read;

use crate::core::prelude::*;

pub fn register(mut c: &mut CollectionPool) {

    direct::register(&mut c);
}
