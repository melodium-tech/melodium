
//! Provides collection of connections types available.

use std::collections::HashMap;
use std::sync::Arc;
use super::descriptor::{ConnectionDescriptor, DataTypeDescriptor, DataTypeStructureDescriptor as DataStructure, DataTypeTypeDescriptor as DataType};

pub struct Connections {
    connections: HashMap<(Option<DataTypeDescriptor>, Option<DataTypeDescriptor>), Arc<ConnectionDescriptor>>
}

impl Connections {

    fn init() -> Self {
        let mut connections = Self {
            connections: HashMap::new()
        };

        // Basic connection, without data transmission.
        connections.insert(ConnectionDescriptor::new(None, None));
        
        // Connections transmitting scalar booleans.
        connections.insert_oi(DataStructure::Scalar, DataType::Boolean, DataStructure::Scalar, DataType::Boolean);
        connections.insert_oi(DataStructure::Scalar, DataType::Boolean, DataStructure::Vector, DataType::Boolean);
        // Connections transmitting vectors of booleans.
        connections.insert_oi(DataStructure::Vector, DataType::Boolean, DataStructure::Vector, DataType::Boolean);

        // Connections transmitting scalar integers.
        connections.insert_oi(DataStructure::Scalar, DataType::Integer, DataStructure::Scalar, DataType::Integer);
        connections.insert_oi(DataStructure::Scalar, DataType::Integer, DataStructure::Vector, DataType::Integer);
        // Connections transmitting vectors of integers.
        connections.insert_oi(DataStructure::Vector, DataType::Integer, DataStructure::Vector, DataType::Integer);

        // Connections transmitting scalar reals.
        connections.insert_oi(DataStructure::Scalar, DataType::Real, DataStructure::Scalar, DataType::Real);
        connections.insert_oi(DataStructure::Scalar, DataType::Real, DataStructure::Vector, DataType::Real);
        // Connections transmitting vectors of reals.
        connections.insert_oi(DataStructure::Vector, DataType::Real, DataStructure::Vector, DataType::Real);

        // Connections transmitting scalar string.
        connections.insert_oi(DataStructure::Scalar, DataType::String, DataStructure::Scalar, DataType::String);
        connections.insert_oi(DataStructure::Scalar, DataType::String, DataStructure::Vector, DataType::String);
        // Connections transmitting vectors of string.
        connections.insert_oi(DataStructure::Vector, DataType::String, DataStructure::Vector, DataType::String);

        // Insert any other possible/legal connections

        connections
    }

    fn singleton() -> &'static Self {
        lazy_static! {
            static ref SINGLETON: Connections = Connections::init();
        }
        &SINGLETON
    }

    fn insert(&mut self, connection: ConnectionDescriptor) {
        self.connections.insert((*connection.output_type(), *connection.input_type()), Arc::new(connection));
    }

    fn insert_oi(&mut self, output_structure: DataStructure, output_type: DataType, input_structure: DataStructure, input_type: DataType) {
        self.insert(ConnectionDescriptor::new(Some(DataTypeDescriptor::new(output_structure, output_type)), Some(DataTypeDescriptor::new(input_structure, input_type))));
    }

    pub fn get(output_type: Option<DataTypeDescriptor>, input_type: Option<DataTypeDescriptor>) -> Option<&'static Arc<ConnectionDescriptor>> {

        Self::singleton().connections.get(&(output_type, input_type))
    }
}
