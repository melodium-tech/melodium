
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

		// Connections transmitting scalar void.
		connections.insert_oi(DataStructure::Scalar, DataType::Void, DataStructure::Scalar, DataType::Void);
		connections.insert_oi(DataStructure::Scalar, DataType::Void, DataStructure::Vector, DataType::Void);
		// Connections transmitting vectors of void.
		connections.insert_oi(DataStructure::Vector, DataType::Void, DataStructure::Vector, DataType::Void);
        
        // Connections transmitting scalar I8.
		connections.insert_oi(DataStructure::Scalar, DataType::I8, DataStructure::Scalar, DataType::I8);
		connections.insert_oi(DataStructure::Scalar, DataType::I8, DataStructure::Vector, DataType::I8);
		// Connections transmitting vectors of I8.
		connections.insert_oi(DataStructure::Vector, DataType::I8, DataStructure::Vector, DataType::I8);


		// Connections transmitting scalar I16.
		connections.insert_oi(DataStructure::Scalar, DataType::I16, DataStructure::Scalar, DataType::I16);
		connections.insert_oi(DataStructure::Scalar, DataType::I16, DataStructure::Vector, DataType::I16);
		// Connections transmitting vectors of I16.
		connections.insert_oi(DataStructure::Vector, DataType::I16, DataStructure::Vector, DataType::I16);


		// Connections transmitting scalar I32.
		connections.insert_oi(DataStructure::Scalar, DataType::I32, DataStructure::Scalar, DataType::I32);
		connections.insert_oi(DataStructure::Scalar, DataType::I32, DataStructure::Vector, DataType::I32);
		// Connections transmitting vectors of I32.
		connections.insert_oi(DataStructure::Vector, DataType::I32, DataStructure::Vector, DataType::I32);


		// Connections transmitting scalar I64.
		connections.insert_oi(DataStructure::Scalar, DataType::I64, DataStructure::Scalar, DataType::I64);
		connections.insert_oi(DataStructure::Scalar, DataType::I64, DataStructure::Vector, DataType::I64);
		// Connections transmitting vectors of I64.
		connections.insert_oi(DataStructure::Vector, DataType::I64, DataStructure::Vector, DataType::I64);


		// Connections transmitting scalar I128.
		connections.insert_oi(DataStructure::Scalar, DataType::I128, DataStructure::Scalar, DataType::I128);
		connections.insert_oi(DataStructure::Scalar, DataType::I128, DataStructure::Vector, DataType::I128);
		// Connections transmitting vectors of I128.
		connections.insert_oi(DataStructure::Vector, DataType::I128, DataStructure::Vector, DataType::I128);


		// Connections transmitting scalar U8.
		connections.insert_oi(DataStructure::Scalar, DataType::U8, DataStructure::Scalar, DataType::U8);
		connections.insert_oi(DataStructure::Scalar, DataType::U8, DataStructure::Vector, DataType::U8);
		// Connections transmitting vectors of U8.
		connections.insert_oi(DataStructure::Vector, DataType::U8, DataStructure::Vector, DataType::U8);


		// Connections transmitting scalar U16.
		connections.insert_oi(DataStructure::Scalar, DataType::U16, DataStructure::Scalar, DataType::U16);
		connections.insert_oi(DataStructure::Scalar, DataType::U16, DataStructure::Vector, DataType::U16);
		// Connections transmitting vectors of U16.
		connections.insert_oi(DataStructure::Vector, DataType::U16, DataStructure::Vector, DataType::U16);


		// Connections transmitting scalar U32.
		connections.insert_oi(DataStructure::Scalar, DataType::U32, DataStructure::Scalar, DataType::U32);
		connections.insert_oi(DataStructure::Scalar, DataType::U32, DataStructure::Vector, DataType::U32);
		// Connections transmitting vectors of U32.
		connections.insert_oi(DataStructure::Vector, DataType::U32, DataStructure::Vector, DataType::U32);


		// Connections transmitting scalar U64.
		connections.insert_oi(DataStructure::Scalar, DataType::U64, DataStructure::Scalar, DataType::U64);
		connections.insert_oi(DataStructure::Scalar, DataType::U64, DataStructure::Vector, DataType::U64);
		// Connections transmitting vectors of U64.
		connections.insert_oi(DataStructure::Vector, DataType::U64, DataStructure::Vector, DataType::U64);


		// Connections transmitting scalar U128.
		connections.insert_oi(DataStructure::Scalar, DataType::U128, DataStructure::Scalar, DataType::U128);
		connections.insert_oi(DataStructure::Scalar, DataType::U128, DataStructure::Vector, DataType::U128);
		// Connections transmitting vectors of U128.
		connections.insert_oi(DataStructure::Vector, DataType::U128, DataStructure::Vector, DataType::U128);


		// Connections transmitting scalar F32.
		connections.insert_oi(DataStructure::Scalar, DataType::F32, DataStructure::Scalar, DataType::F32);
		connections.insert_oi(DataStructure::Scalar, DataType::F32, DataStructure::Vector, DataType::F32);
		// Connections transmitting vectors of F32.
		connections.insert_oi(DataStructure::Vector, DataType::F32, DataStructure::Vector, DataType::F32);


		// Connections transmitting scalar F64.
		connections.insert_oi(DataStructure::Scalar, DataType::F64, DataStructure::Scalar, DataType::F64);
		connections.insert_oi(DataStructure::Scalar, DataType::F64, DataStructure::Vector, DataType::F64);
		// Connections transmitting vectors of F64.
		connections.insert_oi(DataStructure::Vector, DataType::F64, DataStructure::Vector, DataType::F64);


		// Connections transmitting scalar Bool.
		connections.insert_oi(DataStructure::Scalar, DataType::Bool, DataStructure::Scalar, DataType::Bool);
		connections.insert_oi(DataStructure::Scalar, DataType::Bool, DataStructure::Vector, DataType::Bool);
		// Connections transmitting vectors of Bool.
		connections.insert_oi(DataStructure::Vector, DataType::Bool, DataStructure::Vector, DataType::Bool);


		// Connections transmitting scalar Byte.
		connections.insert_oi(DataStructure::Scalar, DataType::Byte, DataStructure::Scalar, DataType::Byte);
		connections.insert_oi(DataStructure::Scalar, DataType::Byte, DataStructure::Vector, DataType::Byte);
		// Connections transmitting vectors of Byte.
		connections.insert_oi(DataStructure::Vector, DataType::Byte, DataStructure::Vector, DataType::Byte);


		// Connections transmitting scalar Char.
		connections.insert_oi(DataStructure::Scalar, DataType::Char, DataStructure::Scalar, DataType::Char);
		connections.insert_oi(DataStructure::Scalar, DataType::Char, DataStructure::Vector, DataType::Char);
		// Connections transmitting vectors of Char.
		connections.insert_oi(DataStructure::Vector, DataType::Char, DataStructure::Vector, DataType::Char);


		// Connections transmitting scalar String.
		connections.insert_oi(DataStructure::Scalar, DataType::String, DataStructure::Scalar, DataType::String);
		connections.insert_oi(DataStructure::Scalar, DataType::String, DataStructure::Vector, DataType::String);
		// Connections transmitting vectors of String.
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
