
use melodium_common::descriptor::{Collection, Identifier, Loader, LoadingError};
use semver::Version;

pub trait Package {
    fn name(&self) -> &str;
    fn version(&self) -> &Version;
    /**
     * Gives all elements that are ready to use as soon as package is loaded in memory.
     * 
     * Those elements are basically the built-in ones, call to this function is relatively cheap.
     */
    fn embedded_collection(&self, loader: &dyn Loader) -> Result<Collection, LoadingError>;
    /**
     * Gives all elements that are contained in the package.
     * 
     * This call trigger disk access, parsing and build of all the elements, which might be costly.
     * It should be used only when other functions in that trait don't fit for usage.
     */
    fn full_collection(&self, loader: &dyn Loader) -> Result<Collection, LoadingError>;
    /**
     * Gives identifiers of all the existing elements in the package.
     * 
     * Call to this function is cheaper than to `full_collection`, but still require some work.
     */
    fn all_identifiers(&self) -> Vec<Identifier>;
    /**
     * Gives the identified element, and the whole other ones it depends on to work.
     * 
     * This function fits for most of the usages, and is the most optimized one for getting functionnal stuff.
     * It loads and build all but only the required elements within the package, wether built-in or to-build elements.
     */
    fn element(&self, loader: &dyn Loader, identifier: &Identifier) -> Result<Collection, LoadingError>;
}
