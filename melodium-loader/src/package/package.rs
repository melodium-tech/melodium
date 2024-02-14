use crate::Loader;
use core::fmt::Debug;
use melodium_common::descriptor::{Collection, Identifier, LoadingResult, PackageRequirement};
use semver::Version;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub enum Package {
    Core(super::CorePackage),
    Raw(super::RawPackage),
    #[cfg(feature = "filesystem")]
    Fs(super::FsPackage),
    #[cfg(feature = "jeu")]
    Jeu(super::JeuPackage),
}

impl PackageInfo for Package {
    fn name(&self) -> &str {
        match self {
            Package::Core(pkg) => pkg.name(),
            Package::Raw(pkg) => pkg.name(),
            #[cfg(feature = "filesystem")]
            Package::Fs(pkg) => pkg.name(),
            #[cfg(feature = "jeu")]
            Package::Jeu(pkg) => pkg.name(),
        }
    }

    fn version(&self) -> &Version {
        match self {
            Package::Core(pkg) => pkg.version(),
            Package::Raw(pkg) => pkg.version(),
            #[cfg(feature = "filesystem")]
            Package::Fs(pkg) => pkg.version(),
            #[cfg(feature = "jeu")]
            Package::Jeu(pkg) => pkg.version(),
        }
    }

    fn requirements(&self) -> &Vec<PackageRequirement> {
        match self {
            Package::Core(pkg) => pkg.requirements(),
            Package::Raw(pkg) => pkg.requirements(),
            #[cfg(feature = "filesystem")]
            Package::Fs(pkg) => pkg.requirements(),
            #[cfg(feature = "jeu")]
            Package::Jeu(pkg) => pkg.requirements(),
        }
    }

    fn entrypoints(&self) -> &HashMap<String, Identifier> {
        match self {
            Package::Core(pkg) => pkg.entrypoints(),
            Package::Raw(pkg) => pkg.entrypoints(),
            #[cfg(feature = "filesystem")]
            Package::Fs(pkg) => pkg.entrypoints(),
            #[cfg(feature = "jeu")]
            Package::Jeu(pkg) => pkg.entrypoints(),
        }
    }
}

impl PackageTrait for Package {
    fn embedded_collection(&self, loader: &Loader) -> LoadingResult<Collection> {
        match self {
            Package::Core(pkg) => pkg.embedded_collection(loader),
            Package::Raw(pkg) => pkg.embedded_collection(loader),
            #[cfg(feature = "filesystem")]
            Package::Fs(pkg) => pkg.embedded_collection(loader),
            #[cfg(feature = "jeu")]
            Package::Jeu(pkg) => pkg.embedded_collection(loader),
        }
    }

    fn full_collection(&self, loader: &Loader) -> LoadingResult<Collection> {
        match self {
            Package::Core(pkg) => pkg.full_collection(loader),
            Package::Raw(pkg) => pkg.full_collection(loader),
            #[cfg(feature = "filesystem")]
            Package::Fs(pkg) => pkg.full_collection(loader),
            #[cfg(feature = "jeu")]
            Package::Jeu(pkg) => pkg.full_collection(loader),
        }
    }

    fn all_identifiers(&self, loader: &Loader) -> LoadingResult<Vec<Identifier>> {
        match self {
            Package::Core(pkg) => pkg.all_identifiers(loader),
            Package::Raw(pkg) => pkg.all_identifiers(loader),
            #[cfg(feature = "filesystem")]
            Package::Fs(pkg) => pkg.all_identifiers(loader),
            #[cfg(feature = "jeu")]
            Package::Jeu(pkg) => pkg.all_identifiers(loader),
        }
    }

    fn element(&self, loader: &Loader, identifier: &Identifier) -> LoadingResult<Collection> {
        match self {
            Package::Core(pkg) => pkg.element(loader, identifier),
            Package::Raw(pkg) => pkg.element(loader, identifier),
            #[cfg(feature = "filesystem")]
            Package::Fs(pkg) => pkg.element(loader, identifier),
            #[cfg(feature = "jeu")]
            Package::Jeu(pkg) => pkg.element(loader, identifier),
        }
    }

    fn make_building(&self, collection: &Arc<Collection>) -> LoadingResult<()> {
        match self {
            Package::Core(pkg) => pkg.make_building(collection),
            Package::Raw(pkg) => pkg.make_building(collection),
            #[cfg(feature = "filesystem")]
            Package::Fs(pkg) => pkg.make_building(collection),
            #[cfg(feature = "jeu")]
            Package::Jeu(pkg) => pkg.make_building(collection),
        }
    }
}

pub trait PackageInfo: Debug {
    fn name(&self) -> &str;
    fn version(&self) -> &Version;
    fn requirements(&self) -> &Vec<PackageRequirement>;
    fn entrypoints(&self) -> &HashMap<String, Identifier>;
}

pub trait PackageTrait: Debug + PackageInfo {
    /**
     * Gives all elements that are ready to use as soon as package is loaded in memory.
     *
     * Those elements are basically the built-in ones, call to this function is relatively cheap.
     */
    fn embedded_collection(&self, loader: &Loader) -> LoadingResult<Collection>;
    /**
     * Gives all elements that are contained in the package.
     *
     * This call trigger disk access and parsing of all the elements, which might be costly.
     * It should be used only when other functions in that trait don't fit for usage.
     */
    fn full_collection(&self, loader: &Loader) -> LoadingResult<Collection>;
    /**
     * Gives identifiers of all the existing elements in the package.
     *
     * Call to this function is cheaper than to `full_collection`, but still require some work.
     */
    fn all_identifiers(&self, loader: &Loader) -> LoadingResult<Vec<Identifier>>;
    /**
     * Gives the identified element, and the whole other ones it depends on to work.
     *
     * This function fits for most of the usages, and is the most optimized one for getting functionnal stuff.
     * It loads and build all but only the required elements within the package, wether built-in or to-build elements.
     */
    fn element(&self, loader: &Loader, identifier: &Identifier) -> LoadingResult<Collection>;
    /**
     * Make the final build of all elements that depends on this package within the given collection.
     *
     * Only after a successful call to this function the elements given by the package are guaranteed to work.
     */
    fn make_building(&self, collection: &Arc<Collection>) -> LoadingResult<()>;
}
