
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
        connections.insert_oi(DataStructure::Scalar, DataType::Boolean, DataStructure::Matrix, DataType::Boolean);
        connections.insert_oi(DataStructure::Scalar, DataType::Boolean, DataStructure::Collection, DataType::Boolean);
        // Connections transmitting vectors of booleans.
        connections.insert_oi(DataStructure::Vector, DataType::Boolean, DataStructure::Vector, DataType::Boolean);
        connections.insert_oi(DataStructure::Vector, DataType::Boolean, DataStructure::Matrix, DataType::Boolean);
        connections.insert_oi(DataStructure::Vector, DataType::Boolean, DataStructure::Collection, DataType::Boolean);
        // Connections transmitting matrix of booleans.
        connections.insert_oi(DataStructure::Matrix, DataType::Boolean, DataStructure::Matrix, DataType::Boolean);
        connections.insert_oi(DataStructure::Matrix, DataType::Boolean, DataStructure::Collection, DataType::Boolean);
        // Connections transmitting collections of booleans.
        connections.insert_oi(DataStructure::Collection, DataType::Boolean, DataStructure::Collection, DataType::Boolean);

        // Connections transmitting scalar integers.
        connections.insert_oi(DataStructure::Scalar, DataType::Integer, DataStructure::Scalar, DataType::Integer);
        connections.insert_oi(DataStructure::Scalar, DataType::Integer, DataStructure::Vector, DataType::Integer);
        connections.insert_oi(DataStructure::Scalar, DataType::Integer, DataStructure::Matrix, DataType::Integer);
        connections.insert_oi(DataStructure::Scalar, DataType::Integer, DataStructure::Collection, DataType::Integer);
        // Connections transmitting vectors of integers.
        connections.insert_oi(DataStructure::Vector, DataType::Integer, DataStructure::Vector, DataType::Integer);
        connections.insert_oi(DataStructure::Vector, DataType::Integer, DataStructure::Matrix, DataType::Integer);
        connections.insert_oi(DataStructure::Vector, DataType::Integer, DataStructure::Collection, DataType::Integer);
        // Connections transmitting matrix of integers.
        connections.insert_oi(DataStructure::Matrix, DataType::Integer, DataStructure::Matrix, DataType::Integer);
        connections.insert_oi(DataStructure::Matrix, DataType::Integer, DataStructure::Collection, DataType::Integer);
        // Connections transmitting collections of integers.
        connections.insert_oi(DataStructure::Collection, DataType::Integer, DataStructure::Collection, DataType::Integer);

        // Connections transmitting scalar reals.
        connections.insert_oi(DataStructure::Scalar, DataType::Real, DataStructure::Scalar, DataType::Real);
        connections.insert_oi(DataStructure::Scalar, DataType::Real, DataStructure::Vector, DataType::Real);
        connections.insert_oi(DataStructure::Scalar, DataType::Real, DataStructure::Matrix, DataType::Real);
        connections.insert_oi(DataStructure::Scalar, DataType::Real, DataStructure::Collection, DataType::Real);
        // Connections transmitting vectors of reals.
        connections.insert_oi(DataStructure::Vector, DataType::Real, DataStructure::Vector, DataType::Real);
        connections.insert_oi(DataStructure::Vector, DataType::Real, DataStructure::Matrix, DataType::Real);
        connections.insert_oi(DataStructure::Vector, DataType::Real, DataStructure::Collection, DataType::Real);
        // Connections transmitting matrix of reals.
        connections.insert_oi(DataStructure::Matrix, DataType::Real, DataStructure::Matrix, DataType::Real);
        connections.insert_oi(DataStructure::Matrix, DataType::Real, DataStructure::Collection, DataType::Real);
        // Connections transmitting collections of reals.
        connections.insert_oi(DataStructure::Collection, DataType::Real, DataStructure::Collection, DataType::Real);

        // Connections transmitting scalar string.
        connections.insert_oi(DataStructure::Scalar, DataType::String, DataStructure::Scalar, DataType::String);
        connections.insert_oi(DataStructure::Scalar, DataType::String, DataStructure::Vector, DataType::String);
        connections.insert_oi(DataStructure::Scalar, DataType::String, DataStructure::Matrix, DataType::String);
        connections.insert_oi(DataStructure::Scalar, DataType::String, DataStructure::Collection, DataType::String);
        // Connections transmitting vectors of string.
        connections.insert_oi(DataStructure::Vector, DataType::String, DataStructure::Vector, DataType::String);
        connections.insert_oi(DataStructure::Vector, DataType::String, DataStructure::Matrix, DataType::String);
        connections.insert_oi(DataStructure::Vector, DataType::String, DataStructure::Collection, DataType::String);
        // Connections transmitting matrix of string.
        connections.insert_oi(DataStructure::Matrix, DataType::String, DataStructure::Matrix, DataType::String);
        connections.insert_oi(DataStructure::Matrix, DataType::String, DataStructure::Collection, DataType::String);
        // Connections transmitting collections of string.
        connections.insert_oi(DataStructure::Collection, DataType::String, DataStructure::Collection, DataType::String);

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
