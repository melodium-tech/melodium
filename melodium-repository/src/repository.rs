use crate::global::Package as PackageDetails;
#[cfg(feature = "network")]
use crate::network::remote;
use crate::technical::{Availability, Element, Package, Platform, PlatformAvailability, Type};
use crate::{RepositoryConfig, RepositoryError, RepositoryResult};
use melodium_common::descriptor::{Version, VersionReq};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Repository {
    config: RepositoryConfig,
    packages: Vec<Package>,
}

impl Repository {
    pub fn new(config: RepositoryConfig) -> Self {
        Self {
            config,
            packages: Vec::new(),
        }
    }

    pub fn config(&self) -> &RepositoryConfig {
        &self.config
    }

    pub fn packages(&self) -> &Vec<Package> {
        &self.packages
    }

    pub fn add_package(&mut self, package: Package) -> RepositoryResult<()> {
        if self.packages.contains(&package) {
            Err(RepositoryError::already_existing_package(
                1,
                package.name,
                package.version,
            ))
        } else {
            self.packages.push(package);
            self.save_packages()
        }
    }

    pub fn load_packages(&mut self) -> RepositoryResult<()> {
        let path = self.packages_path();
        if path.exists() {
            let content =
                fs::read_to_string(path).map_err(|err| RepositoryError::fs_error(23, err))?;
            let packages = serde_json::from_str(&content)
                .map_err(|err| RepositoryError::json_error(24, err))?;
            self.merge_packages(packages)
        } else {
            Ok(())
        }
    }

    pub fn load_packages_with_network(&mut self) -> RepositoryResult<()> {
        #[cfg(not(feature = "network"))]
        {
            Err(RepositoryError::no_network(25))
        }
        #[cfg(feature = "network")]
        {
            let config = self.config.network.clone();
            if let Some(config) = config {
                self.load_packages()?;
                self.merge_packages(remote::get_tech_packages(&config)?)
            } else {
                Err(RepositoryError::no_network(26))
            }
        }
    }

    /**
     * Set the availability of an element of a package for a given platform.
     *
     * If package is from a type that doesn't have platform availability notion, this function return error.
     */
    pub fn set_platform_availability(
        &mut self,
        package: &Package,
        platform: &Platform,
        availability: &Availability,
        element: Element,
    ) -> RepositoryResult<()> {
        if let Some(package) = self.packages.iter_mut().find(|p| *p == package) {
            match &mut package.r#type {
                Type::Compiled {
                    crate_name: _,
                    platforms,
                } => {
                    if let Some(platform_availability) =
                        platforms.iter_mut().find(|pa| &pa.platform == platform)
                    {
                        platform_availability
                            .availability
                            .insert(availability.clone(), element);
                    } else {
                        let pa = PlatformAvailability {
                            platform: platform.clone(),
                            availability: {
                                let mut availabilities = HashMap::new();
                                availabilities.insert(availability.clone(), element);
                                availabilities
                            },
                        };
                        platforms.push(pa);
                    }

                    self.save_packages()
                }
                _ => Err(RepositoryError::not_platform_dependent(
                    18,
                    package.name.clone(),
                    package.version.clone(),
                )),
            }
        } else {
            Err(RepositoryError::unknown_package(
                17,
                package.name.clone(),
                package.version.clone(),
            ))
        }
    }

    /**
     * Get the availability of an element of a package for a given platform.
     */
    pub fn get_platform_availability(
        &self,
        package: &Package,
        platform: &Platform,
        availability: &Availability,
    ) -> RepositoryResult<Element> {
        if let Some(package) = self.packages.iter().find(|p| *p == package) {
            match &package.r#type {
                Type::Compiled {
                    crate_name: _,
                    platforms,
                } => {
                    if let Some(platform_availability) =
                        platforms.iter().find(|pa| &pa.platform == platform)
                    {
                        if let Some(element) = platform_availability.availability.get(&availability)
                        {
                            Ok(element.clone())
                        } else {
                            Err(RepositoryError::platform_unavailable(
                                22,
                                package.name.clone(),
                                package.version.clone(),
                                platform.clone(),
                                availability.clone(),
                            ))
                        }
                    } else {
                        Err(RepositoryError::platform_unavailable(
                            21,
                            package.name.clone(),
                            package.version.clone(),
                            platform.clone(),
                            availability.clone(),
                        ))
                    }
                }
                _ => Err(RepositoryError::not_platform_dependent(
                    19,
                    package.name.clone(),
                    package.version.clone(),
                )),
            }
        } else {
            Err(RepositoryError::unknown_package(
                20,
                package.name.clone(),
                package.version.clone(),
            ))
        }
    }

    /**
     * Get the availability of an element of a package for a given platform, interrogating through network if not available locally.
     */
    #[allow(unused_variables)]
    pub fn get_platform_availability_with_network(
        &mut self,
        package: &Package,
        platform: &Platform,
        availability: &Availability,
    ) -> RepositoryResult<Element> {
        #[cfg(not(feature = "network"))]
        {
            Err(RepositoryError::no_network(36))
        }
        #[cfg(feature = "network")]
        {
            if self.config.network.is_some() {
                if let Ok(element) = self.get_platform_availability(package, platform, availability)
                {
                    Ok(element)
                } else {
                    self.load_packages_with_network()?;
                    self.get_platform_availability(package, platform, availability)
                }
            } else {
                Err(RepositoryError::no_network(37))
            }
        }
    }

    /**
     * Get element of a package for a given platform.
     *
     * This function only checks registered availability, but not if element is really present on disk.
     * See also [reach_platform_element].
     */
    pub fn get_package_element(
        &self,
        package: &Package,
        platform_availability: Option<(&Platform, &Availability)>,
    ) -> RepositoryResult<Element> {
        if let Some(package) = self.packages.iter().find(|p| *p == package) {
            match &package.r#type {
                Type::Compiled {
                    crate_name: _,
                    platforms,
                } => {
                    if let Some((platform, availability)) = platform_availability {
                        if let Some(platform_availability) =
                            platforms.iter().find(|pa| &pa.platform == platform)
                        {
                            if let Some(element) =
                                platform_availability.availability.get(&availability)
                            {
                                Ok(element.clone())
                            } else {
                                Err(RepositoryError::platform_unavailable(
                                    28,
                                    package.name.clone(),
                                    package.version.clone(),
                                    platform.clone(),
                                    availability.clone(),
                                ))
                            }
                        } else {
                            Err(RepositoryError::platform_unavailable(
                                29,
                                package.name.clone(),
                                package.version.clone(),
                                platform.clone(),
                                availability.clone(),
                            ))
                        }
                    } else {
                        Err(RepositoryError::platform_dependent(
                            30,
                            package.name.clone(),
                            package.version.clone(),
                        ))
                    }
                }
                Type::Jeu { file } => Ok(file.clone()),
            }
        } else {
            Err(RepositoryError::unknown_package(
                27,
                package.name.clone(),
                package.version.clone(),
            ))
        }
    }

    /**
     * Get the full path of an element of a package.
     *
     * This function only checks registered availability, but not if element is really present on disk.
     * See also [reach_platform_element].
     */
    pub fn get_package_element_path(
        &self,
        package: &Package,
        platform_availability: Option<(&Platform, &Availability)>,
    ) -> RepositoryResult<PathBuf> {
        let element = self.get_package_element(package, platform_availability)?;
        match &package.r#type {
            Type::Compiled {
                crate_name: _,
                platforms: _,
            } => {
                if let Some((platform, availability)) = platform_availability {
                    let mut path = self.config.repository_location.clone();
                    path.push(package.get_path());
                    path.push(platform.to_string());
                    path.push(availability.to_string());
                    path.push(element.name);
                    Ok(path)
                } else {
                    Err(RepositoryError::platform_dependent(
                        31,
                        package.name.clone(),
                        package.version.clone(),
                    ))
                }
            }
            Type::Jeu { file: _ } => {
                let mut path = self.config.repository_location.clone();
                path.push(package.get_path());
                path.push(element.name);
                Ok(path)
            }
        }
    }

    /**
     * Get the full path of a present element of a package.
     *
     * This function return an error if the element is not present on filesystem.
     */
    pub fn reach_package_element(
        &self,
        package: &Package,
        platform_availability: Option<(&Platform, &Availability)>,
    ) -> RepositoryResult<PathBuf> {
        let path = self.get_package_element_path(package, platform_availability)?;
        if path.is_file() {
            Ok(path)
        } else {
            Err(RepositoryError::package_element_absent(
                32,
                package.name.clone(),
                package.version.clone(),
                platform_availability.map(|(p, a)| (p.clone(), a.clone())),
                self.get_package_element(package, platform_availability)?,
                path,
            ))
        }
    }

    /**
     * Get the full path of a present element of a package for a given platform.
     *
     * This function try to download the element if it is not present on the filesystem.
     */
    #[allow(unused_variables)]
    pub fn reach_package_element_with_network(
        &mut self,
        package: &Package,
        platform_availability: Option<(&Platform, &Availability)>,
    ) -> RepositoryResult<PathBuf> {
        #[cfg(not(feature = "network"))]
        {
            Err(RepositoryError::no_network(38))
        }
        #[cfg(feature = "network")]
        {
            let config = self.config.network.clone();
            if let Some(config) = config {
                let path = self.get_package_element_path(package, platform_availability)?;
                if path.is_file() {
                    Ok(path)
                } else {
                    let element = self.get_package_element(package, platform_availability)?;

                    let url_path = match &package.r#type {
                        Type::Compiled {
                            crate_name: _,
                            platforms: _,
                        } => {
                            if let Some((platform, availability)) = platform_availability {
                                let mut path = package.get_path();
                                path.push(platform.to_string());
                                path.push(availability.to_string());
                                path.push(element.name.clone());
                                path
                            } else {
                                return Err(RepositoryError::platform_dependent(
                                    40,
                                    package.name.clone(),
                                    package.version.clone(),
                                ));
                            }
                        }
                        Type::Jeu { file: _ } => {
                            let mut path = package.get_path();
                            path.push(element.name.clone());
                            path
                        }
                    };

                    remote::get_element_package(&config, element.clone(), url_path, path.clone())?;

                    if path.is_file() {
                        Ok(path)
                    } else {
                        Err(RepositoryError::package_element_absent(
                            41,
                            package.name.clone(),
                            package.version.clone(),
                            platform_availability.map(|(p, a)| (p.clone(), a.clone())),
                            element,
                            path,
                        ))
                    }
                }
            } else {
                Err(RepositoryError::no_network(39))
            }
        }
    }

    pub fn remove_package(&mut self, name: &str, version: &Version) -> RepositoryResult<()> {
        self.packages
            .retain(|pkg| !(pkg.name == name && &pkg.version == version));
        Ok(())
    }

    fn merge_packages(&mut self, packages: Vec<Package>) -> RepositoryResult<()> {
        for package in packages {
            if !self.packages.contains(&package) {
                self.packages.push(package);
            }
        }
        self.save_packages()
    }

    pub fn get_package(
        &self,
        name: &str,
        version_req: &VersionReq,
    ) -> RepositoryResult<Option<Package>> {
        Ok(self
            .packages
            .iter()
            .filter(|p| p.name == name && version_req.matches(&p.version))
            .max_by_key(|p| p.version.clone())
            .map(|p| p.clone()))
    }

    #[allow(unused_variables)]
    pub fn get_package_with_network(
        &mut self,
        name: &str,
        version_req: &VersionReq,
    ) -> RepositoryResult<Option<Package>> {
        #[cfg(not(feature = "network"))]
        {
            Err(RepositoryError::no_network(13))
        }
        #[cfg(feature = "network")]
        {
            let config = self.config.network.clone();
            if let Some(config) = config {
                if let Ok(Some(package)) = self.get_package(name, version_req) {
                    Ok(Some(package))
                } else {
                    let packages = remote::get_tech_packages(&config)?;
                    self.merge_packages(packages)?;
                    self.get_package(name, version_req)
                }
            } else {
                Err(RepositoryError::no_network(14))
            }
        }
    }

    pub fn set_package_details(&self, details: &PackageDetails) -> RepositoryResult<()> {
        if let Some(package) = self
            .packages
            .iter()
            .find(|p| p.name == details.name && p.version == details.version)
        {
            let mut path = self.config.repository_location.clone();
            path.push(package.get_path());
            fs::create_dir_all(path.clone()).map_err(|err| RepositoryError::fs_error(7, err))?;
            path.push("package.json");
            fs::write(
                path,
                serde_json::to_string(&details)
                    .map_err(|err| RepositoryError::json_error(8, err))?,
            )
            .map_err(|err| RepositoryError::fs_error(9, err))
        } else {
            Err(RepositoryError::unknown_package(
                6,
                details.name.clone(),
                details.version.clone(),
            ))
        }
    }

    pub fn get_package_details(&self, package: &Package) -> RepositoryResult<PackageDetails> {
        if let Some(package) = self
            .packages
            .iter()
            .find(|p| p.name == package.name && p.version == package.version)
        {
            let mut path = self.config.repository_location.clone();
            path.push(package.get_path());
            path.push("package.json");
            fs::read_to_string(path)
                .map_err(|err| RepositoryError::fs_error(10, err))
                .and_then(|str| {
                    serde_json::from_str(&str).map_err(|err| RepositoryError::json_error(11, err))
                })
        } else {
            Err(RepositoryError::unknown_package(
                12,
                package.name.to_string(),
                package.version.clone(),
            ))
        }
    }

    #[allow(unused_variables)]
    pub fn get_package_details_with_network(
        &self,
        package: &Package,
    ) -> RepositoryResult<PackageDetails> {
        #[cfg(not(feature = "network"))]
        {
            Err(RepositoryError::no_network(16))
        }
        #[cfg(feature = "network")]
        {
            let config = self.config.network.clone();
            if let Some(config) = config {
                if let Ok(pkg) = self.get_package_details(package) {
                    return Ok(pkg);
                }

                let pkg = remote::get_detail_package(&config, package);
                if let Ok(pkg) = pkg {
                    self.set_package_details(&pkg)?;
                    self.get_package_details(package)
                } else {
                    pkg
                }
            } else {
                Err(RepositoryError::no_network(15))
            }
        }
    }

    fn save_packages(&self) -> RepositoryResult<()> {
        fs::create_dir_all(self.config.repository_location.clone())
            .map_err(|err| RepositoryError::fs_error(3, err))?;
        fs::write(
            self.packages_path(),
            serde_json::to_string(&self.packages)
                .map_err(|err| RepositoryError::json_error(5, err))?,
        )
        .map_err(|err| RepositoryError::fs_error(4, err))
    }

    fn packages_path(&self) -> PathBuf {
        let mut path = self.config.repository_location.clone();
        path.push("packages.json");
        path
    }
}
