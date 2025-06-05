use melodium_common::descriptor::{Identifier, PackageRequirement, Version, VersionReq};
use std::collections::HashMap;

pub fn cicd_pattern(program_name: &str, version: &Version) -> HashMap<String, Vec<u8>> {
    let melodium_version = Version::parse(crate::VERSION).unwrap();

    let compo = melodium_loader::Compo {
        name: program_name.to_string(),
        version: version.clone(),
        requirements: vec![
            PackageRequirement {
                package: "std".into(),
                version_requirement: VersionReq::parse(&if melodium_version.pre.is_empty() {
                    melodium_version.to_string()
                } else {
                    format!("={melodium_version}")
                })
                .unwrap(),
            },
            PackageRequirement {
                package: "log".into(),
                version_requirement: VersionReq::parse(&if melodium_version.pre.is_empty() {
                    melodium_version.to_string()
                } else {
                    format!("={melodium_version}")
                })
                .unwrap(),
            },
            PackageRequirement {
                package: "fs".into(),
                version_requirement: VersionReq::parse(&if melodium_version.pre.is_empty() {
                    melodium_version.to_string()
                } else {
                    format!("={melodium_version}")
                })
                .unwrap(),
            },
            PackageRequirement {
                package: "process".into(),
                version_requirement: VersionReq::parse(&if melodium_version.pre.is_empty() {
                    melodium_version.to_string()
                } else {
                    format!("={melodium_version}")
                })
                .unwrap(),
            },
            PackageRequirement {
                package: "work".into(),
                version_requirement: VersionReq::parse(&if melodium_version.pre.is_empty() {
                    melodium_version.to_string()
                } else {
                    format!("={melodium_version}")
                })
                .unwrap(),
            },
            PackageRequirement {
                package: "distrib".into(),
                version_requirement: VersionReq::parse(&if melodium_version.pre.is_empty() {
                    melodium_version.to_string()
                } else {
                    format!("={melodium_version}")
                })
                .unwrap(),
            },
            PackageRequirement {
                package: "net".into(),
                version_requirement: VersionReq::parse(&if melodium_version.pre.is_empty() {
                    melodium_version.to_string()
                } else {
                    format!("={melodium_version}")
                })
                .unwrap(),
            },
            PackageRequirement {
                package: "http".into(),
                version_requirement: VersionReq::parse(&if melodium_version.pre.is_empty() {
                    melodium_version.to_string()
                } else {
                    format!("={melodium_version}")
                })
                .unwrap(),
            },
            PackageRequirement {
                package: "cicd".into(),
                version_requirement: VersionReq::parse(&if melodium_version.pre.is_empty() {
                    melodium_version.to_string()
                } else {
                    format!("={melodium_version}")
                })
                .unwrap(),
            },
        ],
        entrypoints: {
            let mut entrypoints = HashMap::new();
            entrypoints.insert(
                "main".to_string(),
                Identifier::new(vec![program_name.to_string()], "main"),
            );
            entrypoints.insert(
                "advanced".to_string(),
                Identifier::new(vec![program_name.to_string(), "advanced".to_string()], "main"),
            );
            entrypoints
        },
    }
    .restitute();

    let mut files = HashMap::new();

    files.insert("Compo.toml".to_string(), compo.into());
    files.insert(
        "lib-root.mel".to_string(),
        include_str!("lib-root.mel").into(),
    );
    files.insert("advanced.mel".to_string(), include_str!("advanced.mel").into());

    files
}
