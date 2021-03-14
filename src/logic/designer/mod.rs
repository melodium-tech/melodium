
//! Allow design of logical elements.
//! 
//! Those structs are not aimed to be instancied directly, but through the [elements descriptors](super::descriptor).

pub mod connection;
pub mod model;
pub mod model_instanciation;
pub mod parameter;
pub mod sequence;
pub mod treatment;
pub mod value;

pub use connection::Connection as ConnectionDesigner;
pub use connection::IO as ConnectionIODesigner;
pub use model::Model as ModelDesigner;
pub use model_instanciation::ModelInstanciation as ModelInstanciationDesigner;
pub use parameter::Parameter as ParameterDesigner;
pub use sequence::Sequence as SequenceDesigner;
pub use treatment::Treatment as TreatmentDesigner;
pub use value::Value as ValueDesigner;
