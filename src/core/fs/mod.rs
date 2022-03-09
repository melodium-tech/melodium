

pub mod direct;

pub mod files;
pub mod read_files;
pub mod files_read;
pub mod write_files;

use crate::core::prelude::*;

pub fn register(mut c: &mut CollectionPool) {

    direct::register(&mut c);
}
