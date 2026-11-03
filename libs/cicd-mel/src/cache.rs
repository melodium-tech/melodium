//! Caches of steps, restored before their commands and saved once they succeed.

use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function};

/// Cache of a step: a directory restored from api.melodium.tech before the commands of the
/// step, and saved once they succeed (see `work/cache`).
///
/// It is the directory at `path` of the volume `filesystem` of the step, such as `data`
/// mounted in `/mnt/data` (`cargo` for `/mnt/data/cargo`), or of the local file system of the
/// engine running the step when `filesystem` is none, for steps run by its local executor.
/// It is saved and restored under `name`, unless the step takes the name on its `cache_name`
/// input.
#[mel_data(
    traits (PartialEquality Serialize Deserialize Display ToString)
)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cache {
    pub filesystem: Option<String>,
    pub path: String,
    pub name: String,
}

impl ToString for Cache {
    fn to_string(&self) -> String {
        match &self.filesystem {
            Some(filesystem) => format!("{} ({filesystem}:{})", self.name, self.path),
            None => format!("{} ({})", self.name, self.path),
        }
    }
}

impl Display for Cache {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", ToString::to_string(self))
    }
}

/// Return a cache of the directory at `path` of the volume `filesystem`, under `name`.
///
/// `filesystem` none is the local file system of the engine running the step, for steps run
/// by its local executor (`executor_name` none).
#[mel_function]
pub fn cache(filesystem: Option<string>, path: string, name: string) -> Cache {
    Cache {
        filesystem,
        path,
        name,
    }
}

/// Return the volume of `cache`, none for the local file system or without cache.
#[mel_function]
pub fn cache_filesystem(cache: Option<Cache>) -> Option<string> {
    cache.and_then(|cache| cache.filesystem)
}

/// Return the path of `cache`, none without cache.
#[mel_function]
pub fn cache_path(cache: Option<Cache>) -> Option<string> {
    cache.map(|cache| cache.path)
}

/// Return the name of `cache`, none without cache.
#[mel_function]
pub fn cache_name(cache: Option<Cache>) -> Option<string> {
    cache.map(|cache| cache.name)
}
