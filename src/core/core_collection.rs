
use std::sync::Arc;
use crate::logic::descriptor::{ModelDescriptor, TreatmentDescriptor};
use crate::logic::collection_pool::CollectionPool;

use super::net::tcp_listener::TcpListenerModel;
use super::net::read_tcp_connection::ReadTcpConnectionTreatment;
use super::net::write_tcp_connection::WriteTcpConnectionTreatment;


pub fn core_collection() -> &'static CollectionPool {

    lazy_static! {
        static ref SINGLETON: CollectionPool = {
            let mut c = CollectionPool::new();

            c.models.insert(&(TcpListenerModel::descriptor() as Arc<dyn ModelDescriptor>));

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

            super::arithmetic::add_scalar::register(&mut c);

            super::fs::register(&mut c);

            c.treatments.insert(&(ReadTcpConnectionTreatment::descriptor() as Arc<dyn TreatmentDescriptor>));
            c.treatments.insert(&(WriteTcpConnectionTreatment::descriptor() as Arc<dyn TreatmentDescriptor>));

            c
        };
    }
    &SINGLETON
}

