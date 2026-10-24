//! Storage of runs on Cadence.CI, shared by caches and run artifacts: the volumes of the
//! worker, and transfers with object storage through requests presigned by the API.

#[cfg(feature = "real")]
pub(crate) mod archive;
#[cfg(feature = "real")]
pub(crate) mod transfer;

use core::fmt::{Debug, Formatter};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

/// Request presigned by the API towards object storage, made as given: a URL, a method and
/// headers, never credentials.
///
/// Its URL holds a signature, making it a credential: it never appears in logs, debug
/// events, errors or outputs.
#[derive(Clone, Deserialize)]
pub(crate) struct PresignedRequest {
    pub url: String,
    pub method: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

impl Debug for PresignedRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PresignedRequest")
            .field("method", &self.method)
            .finish_non_exhaustive()
    }
}

/// How an object is uploaded, as the API reserved it.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum Upload {
    /// One request sending the whole object, for objects up to the part size of the API.
    Single { request: PresignedRequest },
    /// One request per part of `part_size` bytes, the last one being shorter, in order.
    Multipart {
        part_size: u64,
        parts: Vec<PresignedRequest>,
    },
}

/// Part of an object uploaded, with the entity tag storage gave it.
#[derive(Debug, Serialize)]
pub(crate) struct UploadedPart {
    pub number: u32,
    pub etag: String,
}

/// Completion of an upload, sent to the API.
#[derive(Debug, Serialize)]
pub(crate) struct Completion {
    pub sha256: String,
    pub parts: Vec<UploadedPart>,
}

/// Gives the directory of `volume` on this worker: listed in `MELODIUM_RUN_VOLUMES`, at the
/// path `MELODIUM_RUN_VOLUME_<volume>` gives.
pub(crate) fn volume_root(volume: &str) -> Result<PathBuf, String> {
    if Ok(true)
        != std::env::var("MELODIUM_RUN_VOLUMES")
            .map(|var| var.split(',').any(|var_volume| var_volume == volume))
    {
        return Err(format!("no volume '{volume}' listed as available"));
    }
    std::env::var_os(format!("MELODIUM_RUN_VOLUME_{volume}"))
        .map(PathBuf::from)
        .ok_or_else(|| format!("no volume '{volume}' available"))
}

/// Gives `path` within `volume`, refusing paths reaching out of it, through `..` or links.
///
/// `path` is relative to the volume, `""` or `"."` being the volume itself.
pub(crate) fn volume_path(volume: &str, path: &str) -> Result<PathBuf, String> {
    let root = volume_root(volume)?;
    let mut full = root.clone();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => full.push(part),
            Component::CurDir => {}
            _ => {
                return Err(format!(
                    "'{path}' is not a path within volume '{volume}', relative and without '..'"
                ))
            }
        }
    }

    // Volumes are shared with executors, whose links could lead the engine elsewhere: the
    // deepest existing part of the path must be within the volume.
    let root = root
        .canonicalize()
        .map_err(|err| format!("volume '{volume}' cannot be reached: {err}"))?;
    let mut existing = full.clone();
    while std::fs::symlink_metadata(&existing).is_err() && existing.pop() {}
    match existing.canonicalize() {
        Ok(reached) if reached.starts_with(&root) => Ok(full),
        _ => Err(format!(
            "'{path}' leads out of volume '{volume}' through a link"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_stay_within_volumes() {
        let volume = std::env::temp_dir().join(format!("work-mel-volume-{}", std::process::id()));
        let outside = std::env::temp_dir().join(format!("work-mel-outside-{}", std::process::id()));
        std::fs::create_dir_all(volume.join("target")).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, volume.join("link")).unwrap();

        std::env::set_var("MELODIUM_RUN_VOLUMES", "other,paths");
        std::env::set_var("MELODIUM_RUN_VOLUME_paths", &volume);

        assert_eq!(
            volume_path("paths", "target").unwrap(),
            volume.join("target")
        );
        assert_eq!(volume_path("paths", "").unwrap(), volume);
        assert_eq!(
            volume_path("paths", "./target/new/dir").unwrap(),
            volume.join("target/new/dir")
        );
        assert!(volume_path("paths", "../etc").is_err());
        assert!(volume_path("paths", "target/../../etc").is_err());
        assert!(volume_path("paths", "/etc").is_err());
        assert!(volume_path("missing", "target").is_err());
        #[cfg(unix)]
        {
            assert!(volume_path("paths", "link").is_err());
            assert!(volume_path("paths", "link/new").is_err());
        }

        std::fs::remove_dir_all(&volume).unwrap();
        std::fs::remove_dir_all(&outside).unwrap();
    }
}
