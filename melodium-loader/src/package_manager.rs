#![allow(unused)]

use crate::package::{Package, PackageTrait};
use crate::{package, PackageInfo};
use melodium_common::descriptor::{
    LoadingError, LoadingResult, Package as CommonPackage, PackageRequirement, VersionReq,
};
use melodium_repository::{
    technical::Availability, technical::Package as RepositoryPackage, technical::Platform,
    Repository,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct PackageManagerConfiguration {
    pub repositories: Vec<Arc<Mutex<Repository>>>,
    pub core_packages: Vec<Arc<dyn CommonPackage>>,
    pub search_locations: Vec<PathBuf>,
    pub raw_elements: Vec<Arc<Vec<u8>>>,
    pub allow_network: bool,
}

#[derive(Debug)]
pub struct PackageManager {
    repositories: Vec<Arc<Mutex<Repository>>>,
    core_packages: Vec<Arc<Package>>,
    search_locations: Vec<PathBuf>,
    found_packages: Mutex<HashMap<PathBuf, Option<Arc<Package>>>>,
    raw_packages: Mutex<HashMap<Arc<Vec<u8>>, Option<Arc<Package>>>>,
    #[allow(unused)]
    allow_network: bool,
}

impl PackageManager {
    pub fn new(config: PackageManagerConfiguration) -> Self {
        Self {
            repositories: config.repositories,
            core_packages: config
                .core_packages
                .into_iter()
                .map(|pkg| Arc::new(Package::Core(package::CorePackage::new(pkg))))
                .collect(),
            search_locations: config.search_locations,
            found_packages: Mutex::new(HashMap::new()),
            raw_packages: Mutex::new(
                config
                    .raw_elements
                    .into_iter()
                    .map(|raw| {
                        (
                            Arc::clone(&raw),
                            Self::raw_to_package(raw).into_result().ok(),
                        )
                    })
                    .collect(),
            ),
            allow_network: config.allow_network,
        }
    }

    fn inspect_locations(&self, name: &str) -> LoadingResult<Vec<Arc<Package>>> {
        #[cfg(feature = "filesystem")]
        {
            let mut result = LoadingResult::new_success(());
            let mut packages = Vec::new();
            let already_inspected_paths = self
                .found_packages
                .lock()
                .unwrap()
                .keys()
                .map(|p| p.clone())
                .collect::<Vec<_>>();

            // Find direct .mel files (path = /home/user/my_script.mel)
            for path in &self.search_locations {
                if !already_inspected_paths.contains(path) {
                    if path.extension() == Some(std::ffi::OsStr::new("mel")) {
                        if let Some(Some(pkg)) =
                            result.merge_degrade_failure(self.inspect_mel_file(path))
                        {
                            if pkg.name() == name {
                                packages.push(pkg);
                            }
                        }
                    }
                }
            }
            // Find direct .jeu files (path = /home/user/my_package.jeu)
            for path in &self.search_locations {
                if !already_inspected_paths.contains(path) {
                    if path.extension() == Some(std::ffi::OsStr::new("jeu")) {
                        if let Some(Some(pkg)) =
                            result.merge_degrade_failure(self.inspect_jeu_file(path))
                        {
                            if pkg.name() == name {
                                packages.push(pkg);
                            }
                        }
                    }
                }
            }

            // Find direct Compo.toml files (path = /home/user/my-package/Compo.toml)
            for path in &self.search_locations {
                if !already_inspected_paths.contains(path) {
                    if path.file_name() == Some(std::ffi::OsStr::new("Compo.toml")) {
                        if let Some(Some(pkg)) =
                            result.merge_degrade_failure(self.inspect_compo_file(path))
                        {
                            if pkg.name() == name {
                                packages.push(pkg);
                            }
                        }
                    }
                }
            }

            // Find direct packages with Compo.toml files (path = /home/user/my-package/)
            for path in &self.search_locations {
                let mut path = path.clone();
                path.push("Compo.toml");
                if !already_inspected_paths.contains(&path) {
                    if path.is_file() {
                        if let Some(Some(pkg)) =
                            result.merge_degrade_failure(self.inspect_compo_file(&path))
                        {
                            if pkg.name() == name {
                                packages.push(pkg);
                            }
                        }
                    }
                }
            }

            // Find .mel subfiles (path = /home/user/bunch_of_mel_files/)
            for path in &self.search_locations {
                if path.is_dir() && {
                    let mut p = path.to_path_buf();
                    p.push("Compo.toml");
                    !p.exists()
                } {
                    if let Some(entries) = result.merge_degrade_failure(
                        glob::glob(&format!("{}/*.mel", path.to_string_lossy()))
                            .map_err(|err| {
                                LoadingError::unreachable_file(
                                    213,
                                    path.clone(),
                                    err.msg.to_string(),
                                )
                            })
                            .into(),
                    ) {
                        for entry in entries {
                            if let Some(entry) = result.merge_degrade_failure(
                                entry
                                    .map_err(|err| {
                                        LoadingError::unreachable_file(
                                            214,
                                            err.path().to_path_buf(),
                                            err.to_string(),
                                        )
                                    })
                                    .into(),
                            ) {
                                if !already_inspected_paths.contains(&entry) {
                                    if let Some(Some(pkg)) =
                                        result.merge_degrade_failure(self.inspect_mel_file(&entry))
                                    {
                                        if pkg.name() == name {
                                            packages.push(pkg);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Find .jeu subfiles (path = /home/user/my_local_dir_with_custom_packages/)
            for path in &self.search_locations {
                if path.is_dir() && {
                    let mut p = path.to_path_buf();
                    p.push("Compo.toml");
                    !p.exists()
                } {
                    if let Some(entries) = result.merge_degrade_failure(
                        glob::glob(&format!("{}/*.jeu", path.to_string_lossy()))
                            .map_err(|err| {
                                LoadingError::unreachable_file(
                                    219,
                                    path.clone(),
                                    err.msg.to_string(),
                                )
                            })
                            .into(),
                    ) {
                        for entry in entries {
                            if let Some(entry) = result.merge_degrade_failure(
                                entry
                                    .map_err(|err| {
                                        LoadingError::unreachable_file(
                                            220,
                                            err.path().to_path_buf(),
                                            err.to_string(),
                                        )
                                    })
                                    .into(),
                            ) {
                                if !already_inspected_paths.contains(&entry) {
                                    if let Some(Some(pkg)) =
                                        result.merge_degrade_failure(self.inspect_jeu_file(&entry))
                                    {
                                        if pkg.name() == name {
                                            packages.push(pkg);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Find subdirectories with Compo.toml (path = /home/user/my_local_dir_with_custom_packages/)
            for path in &self.search_locations {
                if path.is_dir() && {
                    let mut p = path.to_path_buf();
                    p.push("Compo.toml");
                    !p.exists()
                } {
                    if let Some(entries) = result.merge_degrade_failure(
                        glob::glob(&format!("{}/*/Compo.toml", path.to_string_lossy()))
                            .map_err(|err| {
                                LoadingError::unreachable_file(
                                    215,
                                    path.clone(),
                                    err.msg.to_string(),
                                )
                            })
                            .into(),
                    ) {
                        for entry in entries {
                            if let Some(entry) = result.merge_degrade_failure(
                                entry
                                    .map_err(|err| {
                                        LoadingError::unreachable_file(
                                            216,
                                            err.path().to_path_buf(),
                                            err.to_string(),
                                        )
                                    })
                                    .into(),
                            ) {
                                if !already_inspected_paths.contains(&path) {
                                    if let Some(Some(pkg)) = result
                                        .merge_degrade_failure(self.inspect_compo_file(&entry))
                                    {
                                        if pkg.name() == name {
                                            packages.push(pkg);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            LoadingResult::new_success(packages)
        }
        #[cfg(not(feature = "filesystem"))]
        {
            LoadingResult::new_success(Vec::new())
        }
    }

    fn inspect_mel_file(&self, path: &PathBuf) -> LoadingResult<Option<Arc<Package>>> {
        #[cfg(feature = "filesystem")]
        {
            let mut result = LoadingResult::new_success(());
            if let Some(content) = result.merge_degrade_failure(
                std::fs::read(path)
                    .map_err(|err| {
                        LoadingError::unreachable_file(211, path.clone(), err.to_string())
                    })
                    .into(),
            ) {
                if let Some(content) = result.merge_degrade_failure(
                    String::from_utf8(content)
                        .map_err(|err| {
                            LoadingError::content_error(
                                212,
                                Arc::new(crate::content::ContentError::Utf8Error {
                                    path: path.to_string_lossy().to_string(),
                                    error: err.utf8_error(),
                                }),
                            )
                        })
                        .into(),
                ) {
                    if let Some(pkg) =
                        result.merge_degrade_failure(package::RawPackage::new(&content))
                    {
                        let pkg = Arc::new(Package::Raw(pkg));
                        self.found_packages
                            .lock()
                            .unwrap()
                            .insert(path.clone(), Some(Arc::clone(&pkg)));
                        return result.and(LoadingResult::new_success(Some(pkg)));
                    }
                }
            }

            self.found_packages
                .lock()
                .unwrap()
                .insert(path.clone(), None);
            result.and(LoadingResult::new_success(None))
        }

        #[cfg(not(feature = "filesystem"))]
        {
            LoadingResult::new_success(None)
        }
    }

    fn inspect_compo_file(&self, path: &PathBuf) -> LoadingResult<Option<Arc<Package>>> {
        #[cfg(feature = "filesystem")]
        {
            let mut result = LoadingResult::new_success(());

            if let Some(pkg) =
                result.merge_degrade_failure(package::FsPackage::new(path.parent().unwrap_or(path)))
            {
                let pkg = Arc::new(Package::Fs(pkg));
                self.found_packages
                    .lock()
                    .unwrap()
                    .insert(path.clone(), Some(Arc::clone(&pkg)));
                result.and(LoadingResult::new_success(Some(pkg)))
            } else {
                self.found_packages
                    .lock()
                    .unwrap()
                    .insert(path.clone(), None);
                result.and(LoadingResult::new_success(None))
            }
        }
        #[cfg(not(feature = "filesystem"))]
        {
            LoadingResult::new_success(None)
        }
    }

    fn inspect_jeu_file(&self, path: &PathBuf) -> LoadingResult<Option<Arc<Package>>> {
        #[cfg(all(feature = "jeu", feature = "filesystem"))]
        {
            let mut result = LoadingResult::new_success(());
            if let Some(file) = result.merge_degrade_failure(
                std::fs::File::open(path)
                    .map_err(|err| {
                        LoadingError::unreachable_file(237, path.clone(), err.to_string())
                    })
                    .into(),
            ) {
                if let Some(pkg) = result.merge_degrade_failure(package::JeuPackage::new(file)) {
                    let pkg = Arc::new(Package::Jeu(pkg));
                    self.found_packages
                        .lock()
                        .unwrap()
                        .insert(path.clone(), Some(Arc::clone(&pkg)));
                    return result.and(LoadingResult::new_success(Some(pkg)));
                }
            }

            self.found_packages
                .lock()
                .unwrap()
                .insert(path.clone(), None);
            result.and(LoadingResult::new_success(None))
        }

        #[cfg(not(all(feature = "jeu", feature = "filesystem")))]
        {
            LoadingResult::new_success(None)
        }
    }

    fn inspect_library_file(&self, path: &PathBuf) -> LoadingResult<Option<Arc<Package>>> {
        LoadingResult::new_failure(LoadingError::library_loading_error(
            221,
            path.clone(),
            "Library files not supported yet".to_string(),
        ))
    }

    pub fn add_raw_package(&self, raw: Arc<Vec<u8>>) -> LoadingResult<Arc<Package>> {
        let raw_result = Self::raw_to_package(Arc::clone(&raw));
        self.raw_packages
            .lock()
            .unwrap()
            .insert(raw, raw_result.success().map(Arc::clone));
        raw_result
    }

    fn raw_to_package(raw: Arc<Vec<u8>>) -> LoadingResult<Arc<Package>> {
        match String::from_utf8((*raw).clone()) {
            Ok(str) => package::RawPackage::new(&str)
                .and_then(|pkg| LoadingResult::new_success(Arc::new(Package::Raw(pkg)))),
            #[cfg(feature = "jeu")]
            Err(_) => {
                let mut result = package::JeuPackage::new(&**raw)
                    .and_then(|pkg| LoadingResult::new_success(Arc::new(Package::Jeu(pkg))));
                if result.is_failure() {
                    result = result.and_degrade_failure(LoadingResult::new_failure(
                        LoadingError::no_package(
                            210,
                            PackageRequirement::new("[raw package]", &VersionReq::STAR),
                        ),
                    ));
                }
                result
            }
            #[cfg(not(feature = "jeu"))]
            Err(_) => LoadingResult::new_failure(LoadingError::no_package(
                241,
                "[raw package]".to_string(),
            )),
        }
    }

    fn manage_repo_package(
        &self,
        repo: &mut Repository,
        package: RepositoryPackage,
    ) -> LoadingResult<Option<Arc<Package>>> {
        match &package.r#type {
            melodium_repository::technical::Type::Jeu { file: _ } => {
                #[cfg(not(feature = "network"))]
                {
                    match repo.reach_package_element(&package, None) {
                        Ok(path) => self.inspect_jeu_file(&path),
                        Err(err) => LoadingResult::new_failure(LoadingError::repository_error(
                            222,
                            Arc::new(err),
                        )),
                    }
                }
                #[cfg(feature = "network")]
                {
                    match repo.reach_package_element_with_network(&package, None) {
                        Ok(path) => self.inspect_jeu_file(&path),
                        Err(err) => LoadingResult::new_failure(LoadingError::repository_error(
                            221,
                            Arc::new(err),
                        )),
                    }
                }
            }
            melodium_repository::technical::Type::Compiled {
                crate_name: _,
                platforms: _,
            } => {
                let platform = if let Some(platform) = Platform::find(crate::TRIPLE) {
                    platform
                } else {
                    return LoadingResult::new_failure(LoadingError::uncompatible_platform(
                        217,
                        crate::TRIPLE.to_string(),
                    ));
                };
                #[cfg(not(feature = "network"))]
                {
                    match repo
                        .reach_package_element(&package, Some((platform, &Availability::Real)))
                    {
                        Ok(path) => self.inspect_library_file(&path),
                        Err(err) => LoadingResult::new_failure(LoadingError::repository_error(
                            218,
                            Arc::new(err),
                        )),
                    }
                }
                #[cfg(feature = "network")]
                {
                    match repo.reach_package_element_with_network(
                        &package,
                        Some((platform, &Availability::Real)),
                    ) {
                        Ok(path) => self.inspect_library_file(&path),
                        Err(err) => LoadingResult::new_failure(LoadingError::repository_error(
                            223,
                            Arc::new(err),
                        )),
                    }
                }
            }
        }
    }

    pub fn get_packages(&self) -> Vec<Arc<Package>> {
        let mut all_packages = Vec::new();
        all_packages.extend(self.core_packages.iter().map(Arc::clone));
        all_packages.extend(
            self.found_packages
                .lock()
                .unwrap()
                .values()
                .filter_map(|p| p.as_ref().map(Arc::clone)),
        );
        all_packages.extend(
            self.raw_packages
                .lock()
                .unwrap()
                .values()
                .filter_map(|p| p.as_ref().map(Arc::clone)),
        );
        all_packages
    }

    fn retrieve_package(&self, requirement: &PackageRequirement) -> LoadingResult<Arc<Package>> {
        // core_packages
        for pkg in &self.core_packages {
            if pkg.name() == &requirement.package
                && requirement.version_requirement.matches(pkg.version())
            {
                return LoadingResult::new_success(Arc::clone(pkg));
            }
        }

        // raw_elements
        for (_, pkg) in &*self.raw_packages.lock().unwrap() {
            if let Some(pkg) = pkg {
                if pkg.name() == &requirement.package
                    && requirement.version_requirement.matches(pkg.version())
                {
                    return LoadingResult::new_success(Arc::clone(pkg));
                }
            }
        }

        // search_locations
        for (_, pkg) in &*self.found_packages.lock().unwrap() {
            if let Some(pkg) = pkg {
                if pkg.name() == &requirement.package
                    && requirement.version_requirement.matches(pkg.version())
                {
                    return LoadingResult::new_success(Arc::clone(pkg));
                }
            }
        }
        let mut result = LoadingResult::new_success(());
        if let Some(packages) =
            result.merge_degrade_failure(self.inspect_locations(&requirement.package))
        {
            for pkg in packages {
                if pkg.name() == &requirement.package
                    && requirement.version_requirement.matches(pkg.version())
                {
                    return result.and(LoadingResult::new_success(pkg));
                }
            }
        }

        // repositories
        for repo in &self.repositories {
            let mut repo = repo.lock().unwrap();
            #[cfg(not(feature = "network"))]
            {
                match repo.get_package(&requirement.package, &requirement.version_requirement) {
                    Ok(Some(pkg)) => {
                        if let Some(Some(pkg)) =
                            result.merge_degrade_failure(self.manage_repo_package(&mut *repo, pkg))
                        {
                            return result.and(LoadingResult::new_success(pkg));
                        }
                    }
                    Ok(None) => {}
                    Err(err) => {
                        result = result.and_degrade_failure(LoadingResult::new_failure(
                            LoadingError::repository_error(215, Arc::new(err)),
                        ))
                    }
                }
            }
            #[cfg(feature = "network")]
            {
                match repo.get_package_with_network(
                    &requirement.package,
                    &requirement.version_requirement,
                ) {
                    Ok(Some(pkg)) => {
                        if let Some(Some(pkg)) =
                            result.merge_degrade_failure(self.manage_repo_package(&mut *repo, pkg))
                        {
                            return result.and(LoadingResult::new_success(pkg));
                        }
                    }
                    Ok(None) => {}
                    Err(err) => {
                        result = result.and_degrade_failure(LoadingResult::new_failure(
                            LoadingError::repository_error(214, Arc::new(err)),
                        ))
                    }
                }
            }
        }

        result.and_degrade_failure(LoadingResult::new_failure(LoadingError::no_package(
            209,
            requirement.clone(),
        )))
    }

    fn check_package_requirements(&self, package: &Package) -> LoadingResult<()> {
        let mut result = LoadingResult::new_success(());
        for requirement in package.requirements() {
            result.merge_degrade_failure(self.get_package(requirement));
        }
        result
    }

    pub fn get_package(&self, requirement: &PackageRequirement) -> LoadingResult<Arc<Package>> {
        let mut result = self.retrieve_package(requirement);
        if let Some(package) = result.success() {
            result.merge_degrade_failure(self.check_package_requirements(package));
        }
        result
    }
}
