
use melodium_common::descriptor::{Collection, LoadingError, Package};
use melodium_loader::{Loader, LoadingConfig};

pub fn load() -> Result<Collection, LoadingError> {
    let config = LoadingConfig {
        core_packages: core_packages(),
        search_locations: Vec::new(),
    };

    let loader = Loader::new(config);
    loader.full_load()
}

fn core_packages() -> Vec<Box<dyn Package>> {
    let mut packages = Vec::new();
    #[cfg(feature = "conv")]
    packages.push(conv_mel::__mel_package::package());
    #[cfg(feature = "flow")]
    packages.push(flow_mel::__mel_package::package());
    #[cfg(feature = "ops")]
    packages.push(ops_mel::__mel_package::package());
    #[cfg(feature = "type")]
    packages.push(type_mel::__mel_package::package());
    packages
}
