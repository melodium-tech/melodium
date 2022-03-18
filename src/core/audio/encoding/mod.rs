
pub mod wave;

use crate::core::prelude::*;

pub fn register(mut c: &mut CollectionPool) {

    wave::register(&mut c);
}
