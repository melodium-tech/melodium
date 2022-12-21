
use crate::core::prelude::*;
pub mod tcp_listener;

pub fn register(mut c: &mut CollectionPool) {

    tcp_listener::model_host::register(&mut c);
    tcp_listener::read_tcp_connection::register(&mut c);
    tcp_listener::write_tcp_connection::register(&mut c);
}
