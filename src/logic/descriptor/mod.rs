
//! Describe logic elements available.

pub mod buildable;
pub mod connection;
pub mod configured_model;
pub mod context;
pub mod core_model;
pub mod core_treatment;
pub mod datatype;
pub mod designable;
pub mod identified;
pub mod identifier;
pub mod input;
pub mod model;
pub mod output;
pub mod parameter;
pub mod parameterized;
pub mod requirement;
pub mod sequence_treatment;
pub mod treatment;

pub use buildable::Buildable as BuildableDescriptor;
pub use connection::Connection as ConnectionDescriptor;
pub use configured_model::ConfiguredModel as ConfiguredModelDescriptor;
pub use context::Context as ContextDescriptor;
pub use core_model::CoreModel as CoreModelDescriptor;
pub use core_treatment::CoreTreatment as CoreTreatmentDescriptor;
pub use datatype::DataType as DataTypeDescriptor;
pub use datatype::Structure as DataTypeStructureDescriptor;
pub use datatype::Type as DataTypeTypeDescriptor;
pub use designable::Designable as DesignableDescriptor;
pub use identified::Identified as IdentifiedDescriptor;
pub use identifier::Identifier as IdentifierDescriptor;
pub use input::Input as InputDescriptor;
pub use model::Model as ModelDescriptor;
pub use output::Output as OutputDescriptor;
pub use parameter::Parameter as ParameterDescriptor;
pub use parameterized::Parameterized as ParameterizedDescriptor;
pub use sequence_treatment::SequenceTreatment as SequenceTreatmentDescriptor;
pub use treatment::Treatment as TreatmentDescriptor;
