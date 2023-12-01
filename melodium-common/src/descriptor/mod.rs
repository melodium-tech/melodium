//! Descriptive elements.
//!
//! The traits and types present here bring informations about all the components present into Mélodium environment.
//!

pub mod attribuable;
pub mod buildable;
pub mod collection;
pub mod context;
pub mod data_type;
pub mod documented;
pub mod flow;
pub mod function;
pub mod identified;
pub mod identifier;
pub mod input;
pub mod loader;
pub mod model;
pub mod output;
pub mod package;
pub mod parameter;
pub mod parameterized;
pub mod status;
pub mod treatment;
pub mod variability;

pub use attribuable::{Attribute, Attributes, Attribuable};
pub use buildable::{Buildable, ModelBuildMode, TreatmentBuildMode};
pub use collection::{Collection, CollectionTree, Entry};
pub use context::Context;
pub use data_type::{DataType, Structure, Type};
pub use documented::Documented;
pub use flow::Flow;
pub use function::Function;
pub use identified::Identified;
pub use identifier::Identifier;
pub use input::Input;
pub use loader::{
    ContentError, ContentErrors, Loader, LoadingError, LoadingErrors, LoadingResult,
    RepositoryError, RepositoryErrors,
};
pub use model::Model;
pub use output::Output;
pub use package::{Package, PackageRequirement};
pub use parameter::Parameter;
pub use parameterized::{OrderedParameterized, Parameterized};
pub use semver::{Version, VersionReq};
pub use status::Status;
pub use treatment::Treatment;
pub use variability::Variability;
