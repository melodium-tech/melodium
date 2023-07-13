#![allow(unused)]

use core::convert::TryFrom;
use melodium_common::descriptor::{
    Identifier, LoadingError, LoadingResult, PackageRequirement, Version, VersionReq,
};
use toml::{Table, Value};

#[derive(Debug)]
pub struct Compo {
    pub name: String,
    pub version: Version,
    pub requirements: Vec<PackageRequirement>,
    pub main: Option<Identifier>,
}

impl Compo {
    pub fn parse(composition: &str) -> LoadingResult<Self> {
        let composition = match composition.parse::<Table>() {
            Ok(table) => table,
            Err(_) => {
                return LoadingResult::new_failure(LoadingError::wrong_configuration(
                    178,
                    "[unreachable name]".to_string(),
                ))
            }
        };

        if let Some(Value::String(name)) = composition.get("name") {
            let name = name.clone();
            match Version::parse(
                match match composition.get("version") {
                    Some(val) => val,
                    None => {
                        return LoadingResult::new_failure(LoadingError::wrong_configuration(
                            180, name,
                        ))
                    }
                }
                .as_str()
                {
                    Some(val) => val,
                    None => {
                        return LoadingResult::new_failure(LoadingError::wrong_configuration(
                            181, name,
                        ))
                    }
                },
            ) {
                Ok(version) => {
                    let requirements = if let Some(Value::Table(dependencies)) =
                        composition.get("dependencies")
                    {
                        let mut deps = Vec::new();
                        for (package_name, version_req) in dependencies {
                            if let Some(version_req) = version_req.as_str() {
                                if let Ok(version_req) = VersionReq::parse(version_req) {
                                    deps.push(PackageRequirement {
                                        package: package_name.clone(),
                                        version_requirement: version_req,
                                    })
                                } else {
                                    return LoadingResult::new_failure(
                                        LoadingError::wrong_configuration(
                                            207,
                                            package_name.clone(),
                                        ),
                                    );
                                }
                            } else {
                                return LoadingResult::new_failure(
                                    LoadingError::wrong_configuration(206, package_name.clone()),
                                );
                            }
                        }
                        deps
                    } else {
                        Vec::new()
                    };

                    let main = composition
                        .get("main")
                        .and_then(|v| v.as_str())
                        .and_then(|v| Identifier::try_from(v).ok());

                    LoadingResult::new_success(Self {
                        name,
                        version,
                        requirements,
                        main,
                    })
                }
                Err(_) => {
                    return LoadingResult::new_failure(LoadingError::wrong_configuration(227, name))
                }
            }
        } else {
            return LoadingResult::new_failure(LoadingError::wrong_configuration(
                179,
                "[unreachable name]".to_string(),
            ));
        }
    }
}
