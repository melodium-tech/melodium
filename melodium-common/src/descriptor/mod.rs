//! Descriptive elements.
//!
//! The traits and types present here bring informations about all the components present into Mélodium environment.
//!

mod attribuable;
mod buildable;
mod collection;
mod context;
mod data;
mod data_trait;
mod data_type;
mod described_type;
mod documented;
mod flow;
mod function;
mod generic;
mod identified;
mod identifier;
mod identifier_requirement;
mod input;
mod loader;
mod model;
mod output;
mod package;
mod package_requirement;
mod parameter;
mod parameterized;
mod status;
mod treatment;
mod variability;

pub use attribuable::{Attribuable, Attribute, Attributes};
pub use buildable::{Buildable, ModelBuildMode, TreatmentBuildMode};
pub use collection::{Collection, CollectionTree, Entry};
pub use context::Context;
pub use data::Data;
pub use data_trait::DataTrait;
pub use data_type::DataType;
pub use described_type::DescribedType;
pub use documented::Documented;
pub use flow::Flow;
pub use function::Function;
pub use generic::{Generic, Generics};
pub use identified::Identified;
pub use identifier::Identifier;
pub use identifier_requirement::IdentifierRequirement;
pub use input::Input;
pub use loader::{
    ContentError, ContentErrors, Loader, LoadingError, LoadingErrors, LoadingResult,
    RepositoryError, RepositoryErrors,
};
pub use model::Model;
pub use output::Output;
pub use package::Package;
pub use package_requirement::PackageRequirement;
pub use parameter::Parameter;
pub use parameterized::{OrderedParameterized, Parameterized};
pub use semver::{Version, VersionReq};
pub use status::Status;
pub use treatment::Treatment;
pub use variability::Variability;
