//! Descriptive elements.
//!
//! The traits and types present here bring informations about all the components present into Mélodium environment.
//!

pub mod attribuable;
pub mod buildable;
pub mod collection;
pub mod context;
pub mod data_trait;
pub mod data_type;
pub mod described_type;
pub mod documented;
pub mod flow;
pub mod function;
pub mod generic;
pub mod identified;
pub mod identifier;
pub mod input;
pub mod loader;
pub mod model;
pub mod object;
pub mod output;
pub mod package;
pub mod parameter;
pub mod parameterized;
pub mod status;
pub mod treatment;
pub mod variability;

pub use attribuable::{Attribuable, Attribute, Attributes};
pub use buildable::{Buildable, ModelBuildMode, TreatmentBuildMode};
pub use collection::{Collection, CollectionTree, Entry};
pub use context::Context;
pub use data_trait::DataTrait;
pub use data_type::DataType;
pub use described_type::DescribedType;
pub use documented::Documented;
pub use flow::Flow;
pub use function::Function;
pub use generic::{Generic, Generics};
pub use identified::Identified;
pub use identifier::Identifier;
pub use input::Input;
pub use loader::{
    ContentError, ContentErrors, Loader, LoadingError, LoadingErrors, LoadingResult,
    RepositoryError, RepositoryErrors,
};
pub use model::Model;
pub use object::Object;
pub use output::Output;
pub use package::{Package, PackageRequirement};
pub use parameter::Parameter;
pub use parameterized::{OrderedParameterized, Parameterized};
pub use semver::{Version, VersionReq};
pub use status::Status;
pub use treatment::Treatment;
pub use variability::Variability;
