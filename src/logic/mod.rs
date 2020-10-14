
//! Describe and manage logic, provide builders to make logic design.
//! 
//! # Warning
//! This module is currently being designed.

pub mod builder;
pub mod collection;
pub mod collection_pool;
pub mod connections;
pub mod descriptor;
pub mod designer;
pub mod error;

pub use descriptor::CoreTreatmentDescriptor;
pub use descriptor::ConnectionDescriptor;
pub use descriptor::DataTypeDescriptor;
pub use descriptor::DataTypeStructureDescriptor;
pub use descriptor::DataTypeTypeDescriptor;
pub use descriptor::IdentifiedDescriptor;
pub use descriptor::IdentifierDescriptor;
pub use descriptor::InputDescriptor;
pub use descriptor::ModelConfigDescriptor;
pub use descriptor::ModelTypeDescriptor;
pub use descriptor::OutputDescriptor;
pub use descriptor::ParameterDescriptor;
pub use descriptor::SequenceTreatmentDescriptor;
pub use descriptor::TreatmentDescriptor;
