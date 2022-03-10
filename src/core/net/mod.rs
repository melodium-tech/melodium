
use crate::core::prelude::*;
pub mod tcp_listener;
pub mod read_tcp_connection;
pub mod write_tcp_connection;

pub fn register(mut c: &mut CollectionPool) {

    c.models.insert(&(tcp_listener::TcpListenerModel::descriptor() as Arc<dyn ModelDescriptor>));

    read_tcp_connection::read_tcp_connection::register(&mut c);
    write_tcp_connection::write_tcp_connection::register(&mut c);
}
