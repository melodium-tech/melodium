#![allow(unused)]

use core::convert::TryFrom;
use melodium_common::descriptor::{
    Identifier, LoadingError, LoadingResult, PackageRequirement, Version, VersionReq,
};
use std::collections::HashMap;
use toml::{Table, Value};

#[derive(Debug)]
pub struct Compo {
    pub name: String,
    pub version: Version,
    pub requirements: Vec<PackageRequirement>,
    pub entrypoints: HashMap<String, Identifier>,
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

                    let mut entrypoints = HashMap::new();
                    if let Some(Value::Table(toml_entrypoints)) = composition.get("entrypoints") {
                        for (name, pos_id) in toml_entrypoints {
                            if let Value::String(pos_id) = pos_id {
                                if let Ok(pos_id) = Identifier::try_from(pos_id) {
                                    entrypoints.insert(name.clone(), pos_id);
                                } else {
                                    return LoadingResult::new_failure(
                                        LoadingError::wrong_configuration(242, name.clone()),
                                    );
                                }
                            }
                        }
                    }

                    LoadingResult::new_success(Self {
                        name,
                        version,
                        requirements,
                        entrypoints,
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

    pub fn restitute(&self) -> String {
        let mut toml = Table::new();

        toml.insert("name".to_string(), Value::String(self.name.clone()));
        toml.insert(
            "version".to_string(),
            Value::String(self.version.to_string()),
        );
        if !self.requirements.is_empty() {
            let mut deps = Table::new();
            for requirement in &self.requirements {
                deps.insert(
                    requirement.package.clone(),
                    Value::String(requirement.version_requirement.to_string()),
                );
            }
            toml.insert("dependencies".to_string(), Value::Table(deps));
        }

        if !self.entrypoints.is_empty() {
            let mut entrypoints = Table::new();
            for (name, id) in &self.entrypoints {
                entrypoints.insert(name.clone(), Value::String(id.to_string()));
            }
            toml.insert("entrypoints".to_string(), Value::Table(entrypoints));
        }

        toml::to_string_pretty(&toml).unwrap()
    }
}
