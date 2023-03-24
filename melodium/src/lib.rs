
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
    #[cfg(feature = "conv-mel")]
    packages.push(conv_mel::__mel_package::package());
    #[cfg(feature = "flow-mel")]
    packages.push(flow_mel::__mel_package::package());
    #[cfg(feature = "fs-mel")]
    packages.push(fs_mel::__mel_package::package());
    #[cfg(feature = "ops-mel")]
    packages.push(ops_mel::__mel_package::package());
    #[cfg(feature = "type-mel")]
    packages.push(type_mel::__mel_package::package());
    packages
}
