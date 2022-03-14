
use std::sync::Arc;
use crate::logic::descriptor::{ModelDescriptor, TreatmentDescriptor};
use crate::logic::collection_pool::CollectionPool;

pub fn core_collection() -> &'static CollectionPool {

    lazy_static! {
        static ref SINGLETON: CollectionPool = {
            let mut c = CollectionPool::new();

            super::generation::scalar_generator::register(&mut c);

            super::cast::lossless_scalar::register(&mut c);
            super::cast::lossless_vector::register(&mut c);
            super::cast::lossy_scalar::register(&mut c);
            super::cast::lossy_vector::register(&mut c);

            super::conversion::scalar_to_byte::register(&mut c);
            super::conversion::scalar_to_string::register(&mut c);
            super::conversion::vector_to_string::register(&mut c);
            super::conversion::scalar_float_to_integer::register(&mut c);
            super::conversion::vector_float_to_integer::register(&mut c);

            super::flow::block_to_stream::register(&mut c);
            super::flow::stream_to_block::register(&mut c);

            super::arithmetic::add_scalar::register(&mut c);

            super::text::bytes_to_string::register(&mut c);

            super::fs::register(&mut c);

            super::net::register(&mut c);

            super::audio::register(&mut c);

            c
        };
    }
    &SINGLETON
}


