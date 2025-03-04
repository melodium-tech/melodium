use melodium_common::descriptor::{PackageRequirement, Version, VersionReq};
use std::collections::HashMap;

pub fn raw_pattern(program_name: &str, version: &Version) -> HashMap<String, Vec<u8>> {
    let melodium_version = Version::parse(crate::VERSION).unwrap();

    let compo = melodium_loader::Compo {
        name: program_name.to_string(),
        version: version.clone(),
        requirements: vec![PackageRequirement {
            package: "std".into(),
            version_requirement: VersionReq::parse(&if melodium_version.pre.is_empty() {
                melodium_version.to_string()
            } else {
                format!("={melodium_version}")
            })
            .unwrap(),
        }],
        entrypoints: HashMap::new(),
    }
    .restitute();

    let root = format!("");

    let mut files = HashMap::new();

    files.insert("Compo.toml".to_string(), compo.into());
    files.insert("lib-root.mel".to_string(), root.into());

    files
}
