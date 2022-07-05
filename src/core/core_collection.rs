
use crate::logic::collection_pool::CollectionPool;

pub fn core_collection() -> &'static CollectionPool {

    lazy_static! {
        static ref SINGLETON: CollectionPool = {
            let mut c = CollectionPool::new();

            super::engine::register(&mut c);

            super::filling::register(&mut c);

            super::logic::register(&mut c);

            super::generation::void_generator::register(&mut c);

            super::cast::lossless_scalar::register(&mut c);
            super::cast::lossy_scalar::register(&mut c);

            super::conversion::scalar_to_void::register(&mut c);
            super::conversion::vector_to_void::register(&mut c);
            super::conversion::scalar_to_byte::register(&mut c);
            super::conversion::scalar_to_string::register(&mut c);
            super::conversion::scalar_float_to_integer::register(&mut c);
            super::conversion::scalar_float::register(&mut c);

            super::flow::trigger::trigger::register(&mut c);
            super::flow::linearize::register(&mut c);
            super::flow::organizer::register(&mut c);
            super::flow::block_to_stream::register(&mut c);
            super::flow::stream_to_block::register(&mut c);

            super::arithmetic::implementation::register(&mut c);
            super::trigonometry::register(&mut c);

            super::comparison::numbers::register(&mut c);

            super::text::bytes_to_string::register(&mut c);

            super::fs::register(&mut c);

            super::net::register(&mut c);

            super::audio::register(&mut c);

            c
        };
    }
    &SINGLETON
}


