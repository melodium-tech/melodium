use melodium_common::descriptor::{PackageRequirement, Version, VersionReq};
use melodium_repository::{
    global::{Author, Package as PackageDetails, Tag},
    network::NetworkRepositoryConfiguration,
    technical::{Availability, Element, Package, Platform, PlatformAvailability, Type},
    Repository, RepositoryConfig,
};

#[test]
fn get_remote_packages() {
    let config = RepositoryConfig {
        repository_location: "/tmp/repo".into(),
        network: Some(NetworkRepositoryConfiguration::default()),
    };
    let mut repo = Repository::new(config);

    repo.load_packages_with_network().unwrap();
}

#[test]
fn make_basic_repo() {
    let config = RepositoryConfig {
        repository_location: "/tmp/repo".into(),
        network: None,
    };
    let mut repo = Repository::new(config);

    let conv_package = Package {
        name: "conv".to_string(),
        version: Version::parse("0.7.0").unwrap(),
        requirements: vec![],
        r#type: Type::Compiled {
            crate_name: "conv-mel".to_string(),
            platforms: vec![PlatformAvailability {
                platform: Platform::find("x86_64-unknown-linux-gnu").unwrap().clone(),
                availability: vec![(
                    Availability::Real,
                    Element {
                        name: "libconv_mel.so".to_string(),
                        checksum:
                            "a582bec535717ccc76753b8ab329c98cd16f9495ef9d14d762a75a4cb17b9edf"
                                .to_string(),
                    },
                )]
                .into_iter()
                .collect(),
            }],
        },
    };

    let mut conv_details = PackageDetails::from(&conv_package);
    conv_details.authors = vec![Author::new("Quentin VIGNAUD")];
    conv_details.description.insert(
        "en".to_string(),
        "Mélodium core types conversion library".to_string(),
    );
    conv_details.license = "EUPL".to_string();
    conv_details.homepage = Some("https://melodium.tech/".to_string());
    conv_details.repository = Some("https://gitlab.com/melodium/melodium".to_string());
    conv_details.tags = vec![Tag::Std];

    let engine_package = Package {
        name: "engine".to_string(),
        version: Version::parse("0.7.0").unwrap(),
        requirements: vec![],
        r#type: Type::Compiled {
            crate_name: "engine-mel".to_string(),
            platforms: Vec::new(),
        },
    };

    let flow_package = Package {
        name: "flow".to_string(),
        version: Version::parse("0.7.0").unwrap(),
        requirements: vec![PackageRequirement {
            package: "conv".to_string(),
            version_requirement: VersionReq::parse("=0.7.0").unwrap(),
        }],
        r#type: Type::Compiled {
            crate_name: "flow-mel".to_string(),
            platforms: Vec::new(),
        },
    };

    let fs_package = Package {
        name: "fs".to_string(),
        version: Version::parse("0.7.0").unwrap(),
        requirements: vec![],
        r#type: Type::Compiled {
            crate_name: "fs-mel".to_string(),
            platforms: Vec::new(),
        },
    };

    let ops_package = Package {
        name: "ops".to_string(),
        version: Version::parse("0.7.0").unwrap(),
        requirements: vec![],
        r#type: Type::Compiled {
            crate_name: "ops-mel".to_string(),
            platforms: Vec::new(),
        },
    };

    let type_package = Package {
        name: "type".to_string(),
        version: Version::parse("0.7.0").unwrap(),
        requirements: vec![],
        r#type: Type::Compiled {
            crate_name: "type-mel".to_string(),
            platforms: Vec::new(),
        },
    };

    repo.add_package(conv_package).unwrap();
    repo.set_package_details(&conv_details).unwrap();
    repo.add_package(engine_package).unwrap();
    repo.add_package(flow_package).unwrap();
    repo.add_package(fs_package).unwrap();
    repo.add_package(ops_package).unwrap();
    repo.add_package(type_package).unwrap();
}
