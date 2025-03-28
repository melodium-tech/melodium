mod context;
mod data;
mod function;
mod model;
mod source;
mod treatment;

pub use context::Context;
pub use data::Data;
pub use function::Function;
pub use model::Model;
pub use source::Source;
pub use treatment::Treatment;

pub fn module_path_to_identifier(
    version: &str,
    path: &str,
    name: &str,
) -> crate::common::descriptor::Identifier {
    let mut path = path.split("::").map(|s| s.to_string()).collect::<Vec<_>>();
    path.remove(path.len() - 1);
    path[0] = path[0]
        .strip_suffix("_mel")
        .map(|s| s.to_string())
        .unwrap_or_else(|| path[0].clone());
    crate::common::descriptor::Identifier::new_versionned(
        &crate::common::descriptor::Version::parse(version).unwrap(),
        path,
        name,
    )
}
