
mod connection;
mod core_treatment;
mod datatype;
mod identified;
mod identifier;
mod input;
mod model_config;
mod model_type;
mod output;
mod parameter;
mod requirement;
mod sequence_treatment;
mod treatment;

pub use connection::Connection as ConnectionDescriptor;
pub use core_treatment::CoreTreatment as CoreTreatmentDescriptor;
pub use datatype::DataType as DataTypeDescriptor;
pub use datatype::Structure as DataTypeStructureDescriptor;
pub use datatype::Type as DataTypeTypeDescriptor;
pub use identified::Identified as IdentifiedDescriptor;
pub use identifier::Identifier as IdentifierDescriptor;
pub use input::Input as InputDescriptor;
pub use model_config::ModelConfig as ModelConfigDescriptor;
pub use model_type::ModelType as ModelTypeDescriptor;
pub use output::Output as OutputDescriptor;
pub use parameter::Parameter as ParameterDescriptor;
pub use sequence_treatment::SequenceTreatment as SequenceTreatmentDescriptor;
pub use treatment::Treatment as TreatmentDescriptor;
